//! Broken circuit-breaker: pause/unpause does not set or check a paused flag.
//!
//! A `pub fn pause` that does not write a paused/halted flag to storage has no
//! effect. State-mutating functions that do not read a paused flag before writing
//! allow the contract to operate even when it should be halted.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ExprMethodCall, File, Visibility};

const CHECK_NAME: &str = "broken-pause";

fn receiver_chain_contains_storage(expr: &Expr) -> bool {
    match expr {
        Expr::MethodCall(m) => {
            if m.method == "storage" {
                return true;
            }
            receiver_chain_contains_storage(&m.receiver)
        }
        Expr::Field(f) => receiver_chain_contains_storage(&f.base),
        _ => false,
    }
}

fn extract_key_str(arg: &Expr) -> String {
    let inner = match arg {
        Expr::Reference(r) => &*r.expr,
        other => other,
    };
    match inner {
        Expr::Path(p) => p
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default(),
        Expr::Lit(l) => match &l.lit {
            syn::Lit::Str(s) => s.value(),
            _ => String::new(),
        },
        Expr::Macro(m) => m
            .mac
            .tokens
            .to_string()
            .trim()
            .trim_matches('"')
            .to_string(),
        _ => String::new(),
    }
}

fn key_looks_like_paused(key: &str) -> bool {
    let lower = key.to_lowercase();
    lower.contains("pause")
        || lower.contains("halt")
        || lower.contains("frozen")
        || lower.contains("stopped")
        || lower.contains("emergency")
}

#[derive(Default)]
struct PauseScan {
    writes_paused_flag: bool,
    reads_paused_flag: bool,
    has_storage_set: bool,
    first_set_line: usize,
    set_seen: bool,
}

impl<'ast> Visit<'ast> for PauseScan {
    fn visit_expr_method_call(&mut self, i: &ExprMethodCall) {
        let method = i.method.to_string();
        if method == "set" && receiver_chain_contains_storage(&i.receiver) {
            if !self.set_seen {
                self.has_storage_set = true;
                self.set_seen = true;
                self.first_set_line = i.span().start().line;
            }
            if let Some(arg) = i.args.first() {
                if key_looks_like_paused(&extract_key_str(arg)) {
                    self.writes_paused_flag = true;
                }
            }
        }
        if matches!(method.as_str(), "get" | "has" | "get_unchecked")
            && receiver_chain_contains_storage(&i.receiver)
        {
            if let Some(arg) = i.args.first() {
                if key_looks_like_paused(&extract_key_str(arg)) {
                    self.reads_paused_flag = true;
                }
            }
        }
        visit::visit_expr_method_call(self, i);
    }
}

pub struct BrokenPauseCheck;

impl Check for BrokenPauseCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        for method in contractimpl_functions(file) {
            if !matches!(method.vis, Visibility::Public(_)) {
                continue;
            }
            let name = method.sig.ident.to_string();
            let mut scan = PauseScan::default();
            scan.visit_block(&method.block);

            // Rule 1: pub fn pause / unpause must write a paused flag.
            if (name == "pause" || name == "unpause" || name == "emergency_stop")
                && !scan.writes_paused_flag
            {
                out.push(Finding {
                    check_name: CHECK_NAME.to_string(),
                    severity: Severity::High,
                    file_path: String::new(),
                    line: method.sig.fn_token.span().start().line,
                    function_name: name.clone(),
                    description: format!(
                        "Method `{name}` does not write a paused/halted flag to storage. \
                         The circuit-breaker has no effect — the contract state is unchanged \
                         when `{name}` is called. Write a boolean flag under a key like \
                         `paused` to storage."
                    ),
                });
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Check;
    use syn::parse_file;

    #[test]
    fn flags_pause_without_flag_write() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn pause(env: Env) {
        // BUG: does nothing — no paused flag written
    }
}
"#;
        let file = parse_file(src)?;
        let hits = BrokenPauseCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].severity, Severity::High);
        assert!(hits[0].description.contains("paused/halted flag"));
        Ok(())
    }

    #[test]
    fn no_finding_when_pause_writes_flag() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, symbol_short, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn pause(env: Env) {
        env.require_auth();
        env.storage().instance().set(&symbol_short!("paused"), &true);
    }
}
"#;
        let file = parse_file(src)?;
        let hits = BrokenPauseCheck.run(&file, "");
        assert!(hits.is_empty(), "{hits:?}");
        Ok(())
    }

    #[test]
    fn flags_unpause_without_flag_write() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn unpause(env: Env) {}
}
"#;
        let file = parse_file(src)?;
        let hits = BrokenPauseCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        Ok(())
    }

    #[test]
    fn no_finding_for_non_pause_fn() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn transfer(env: Env) {}
}
"#;
        let file = parse_file(src)?;
        let hits = BrokenPauseCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }
}
