//! `env.storage().temporary().set(key, val)` without a matching `extend_ttl` in the same function.
//!
//! Temporary entries written without an explicit TTL extension use the default
//! (potentially very short) TTL. Locks, nonces, and other short-lived state
//! must have their TTL explicitly set to match the intended validity window.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ExprMethodCall, File};

const CHECK_NAME: &str = "temp-set-no-ttl";

fn receiver_chain_contains_temporary(expr: &Expr) -> bool {
    match expr {
        Expr::MethodCall(m) => {
            if m.method == "temporary" {
                return true;
            }
            receiver_chain_contains_temporary(&m.receiver)
        }
        Expr::Field(f) => receiver_chain_contains_temporary(&f.base),
        _ => false,
    }
}

fn extract_first_arg_str(m: &ExprMethodCall) -> String {
    let Some(arg) = m.args.first() else {
        return String::new();
    };
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
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default(),
        _ => String::new(),
    }
}

#[derive(Default)]
struct TempSetEntry {
    key: String,
    line: usize,
}

#[derive(Default)]
struct TempTtlScan {
    sets: Vec<TempSetEntry>,
    extend_ttl_keys: Vec<String>,
}

impl<'ast> Visit<'ast> for TempTtlScan {
    fn visit_expr_method_call(&mut self, i: &ExprMethodCall) {
        if receiver_chain_contains_temporary(&i.receiver) {
            let method = i.method.to_string();
            if method == "set" {
                self.sets.push(TempSetEntry {
                    key: extract_first_arg_str(i),
                    line: i.span().start().line,
                });
            } else if matches!(method.as_str(), "extend_ttl" | "bump") {
                self.extend_ttl_keys.push(extract_first_arg_str(i));
            }
        }
        visit::visit_expr_method_call(self, i);
    }
}

pub struct TempSetNoTtlCheck;

impl Check for TempSetNoTtlCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        for method in contractimpl_functions(file) {
            let fn_name = method.sig.ident.to_string();
            let mut scan = TempTtlScan::default();
            scan.visit_block(&method.block);

            for entry in &scan.sets {
                // Flag if no extend_ttl for the same key exists in this function.
                let has_ttl = scan.extend_ttl_keys.iter().any(|k| {
                    // Match by key name, or accept a wildcard (empty key = unknown).
                    k == &entry.key || k.is_empty() || entry.key.is_empty()
                });
                if !has_ttl {
                    out.push(Finding {
                        check_name: CHECK_NAME.to_string(),
                        severity: Severity::Low,
                        file_path: String::new(),
                        line: entry.line,
                        function_name: fn_name.clone(),
                        description: format!(
                            "Method `{fn_name}` writes to `env.storage().temporary()` (key: \
                             `{}`) without calling `extend_ttl` in the same function. The \
                             entry will use the default TTL, which may be too short for the \
                             intended validity window. Call `extend_ttl` immediately after \
                             `set` to explicitly control the entry lifetime.",
                            if entry.key.is_empty() {
                                "<unknown>"
                            } else {
                                &entry.key
                            }
                        ),
                    });
                }
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
    fn flags_temp_set_without_extend_ttl() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, symbol_short, Env};
pub struct C;
const LOCK: soroban_sdk::Symbol = symbol_short!("lock");
#[contractimpl]
impl C {
    pub fn acquire_lock(env: Env) {
        // BUG: no extend_ttl after set
        env.storage().temporary().set(&LOCK, &true);
    }
}
"#;
        let file = parse_file(src)?;
        let hits = TempSetNoTtlCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].severity, Severity::Low);
        assert!(hits[0].description.contains("extend_ttl"));
        Ok(())
    }

    #[test]
    fn no_finding_when_extend_ttl_present() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, symbol_short, Env};
pub struct C;
const LOCK: soroban_sdk::Symbol = symbol_short!("lock");
#[contractimpl]
impl C {
    pub fn acquire_lock(env: Env) {
        env.storage().temporary().set(&LOCK, &true);
        env.storage().temporary().extend_ttl(&LOCK, 100, 200);
    }
}
"#;
        let file = parse_file(src)?;
        let hits = TempSetNoTtlCheck.run(&file, "");
        assert!(hits.is_empty(), "{hits:?}");
        Ok(())
    }

    #[test]
    fn no_finding_for_persistent_set() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, symbol_short, Env};
pub struct C;
const KEY: soroban_sdk::Symbol = symbol_short!("k");
#[contractimpl]
impl C {
    pub fn store(env: Env, val: u32) {
        env.storage().persistent().set(&KEY, &val);
    }
}
"#;
        let file = parse_file(src)?;
        let hits = TempSetNoTtlCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }

    #[test]
    fn flags_multiple_temp_sets_without_ttl() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, symbol_short, Env};
pub struct C;
const A: soroban_sdk::Symbol = symbol_short!("a");
const B: soroban_sdk::Symbol = symbol_short!("b");
#[contractimpl]
impl C {
    pub fn store_both(env: Env) {
        env.storage().temporary().set(&A, &1u32);
        env.storage().temporary().set(&B, &2u32);
    }
}
"#;
        let file = parse_file(src)?;
        let hits = TempSetNoTtlCheck.run(&file, "");
        assert_eq!(hits.len(), 2);
        Ok(())
    }
}
