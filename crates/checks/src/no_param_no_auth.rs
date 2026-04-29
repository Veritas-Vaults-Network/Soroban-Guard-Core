//! Public zero-parameter functions that write to storage without auth.
//!
//! A `pub fn` with no non-Env parameters and no `require_auth` call that
//! writes to storage is callable by anyone. These are often forgotten utility
//! functions that should be private or removed.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ExprMethodCall, File, FnArg, Type, Visibility};

const CHECK_NAME: &str = "no-param-no-auth";

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

fn type_last_ident(ty: &Type) -> String {
    match ty {
        Type::Path(p) => p
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default(),
        Type::Reference(r) => type_last_ident(&r.elem),
        _ => String::new(),
    }
}

fn is_env_param(arg: &FnArg) -> bool {
    match arg {
        FnArg::Typed(pt) => {
            let ty = type_last_ident(&pt.ty);
            ty == "Env"
        }
        FnArg::Receiver(_) => false,
    }
}

fn has_only_env_params(method: &syn::ImplItemFn) -> bool {
    method
        .sig
        .inputs
        .iter()
        .all(|arg| matches!(arg, FnArg::Receiver(_)) || is_env_param(arg))
}

#[derive(Default)]
struct BodyScan {
    has_storage_set: bool,
    has_require_auth: bool,
    first_set_line: usize,
}

impl<'ast> Visit<'ast> for BodyScan {
    fn visit_expr_method_call(&mut self, i: &ExprMethodCall) {
        let method = i.method.to_string();
        if method == "set"
            && receiver_chain_contains_storage(&i.receiver)
            && self.first_set_line == 0
        {
            self.has_storage_set = true;
            self.first_set_line = i.span().start().line;
        }
        if matches!(method.as_str(), "require_auth" | "require_auth_for_args") {
            self.has_require_auth = true;
        }
        visit::visit_expr_method_call(self, i);
    }
}

pub struct NoParamNoAuthCheck;

impl Check for NoParamNoAuthCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        for method in contractimpl_functions(file) {
            if !matches!(method.vis, Visibility::Public(_)) {
                continue;
            }
            if !has_only_env_params(method) {
                continue;
            }
            let mut scan = BodyScan::default();
            scan.visit_block(&method.block);
            if !scan.has_storage_set || scan.has_require_auth {
                continue;
            }
            let fn_name = method.sig.ident.to_string();
            let line = if scan.first_set_line > 0 {
                scan.first_set_line
            } else {
                method.sig.fn_token.span().start().line
            };
            out.push(Finding {
                check_name: CHECK_NAME.to_string(),
                severity: Severity::Medium,
                file_path: String::new(),
                line,
                function_name: fn_name.clone(),
                description: format!(
                    "Public method `{fn_name}` has no non-Env parameters and no \
                     `require_auth()` call, but writes to storage. Any caller can invoke \
                     this function and mutate contract state. Make it private, add \
                     parameters, or add an authorization check."
                ),
            });
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
    fn flags_zero_param_storage_write_no_auth() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, symbol_short, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn reset(env: Env) {
        env.storage().instance().set(&symbol_short!("count"), &0u32);
    }
}
"#;
        let file = parse_file(src)?;
        let hits = NoParamNoAuthCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].severity, Severity::Medium);
        assert!(hits[0].description.contains("no non-Env parameters"));
        Ok(())
    }

    #[test]
    fn no_finding_when_has_extra_param() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, symbol_short, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn reset(env: Env, val: u32) {
        env.storage().instance().set(&symbol_short!("count"), &val);
    }
}
"#;
        let file = parse_file(src)?;
        let hits = NoParamNoAuthCheck.run(&file, "");
        assert!(hits.is_empty(), "{hits:?}");
        Ok(())
    }

    #[test]
    fn no_finding_when_auth_present() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, symbol_short, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn reset(env: Env) {
        env.require_auth();
        env.storage().instance().set(&symbol_short!("count"), &0u32);
    }
}
"#;
        let file = parse_file(src)?;
        let hits = NoParamNoAuthCheck.run(&file, "");
        assert!(hits.is_empty(), "{hits:?}");
        Ok(())
    }

    #[test]
    fn no_finding_for_private_fn() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, symbol_short, Env};
pub struct C;
#[contractimpl]
impl C {
    fn reset(env: Env) {
        env.storage().instance().set(&symbol_short!("count"), &0u32);
    }
}
"#;
        let file = parse_file(src)?;
        let hits = NoParamNoAuthCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }

    #[test]
    fn no_finding_when_no_storage_write() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, symbol_short, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn read(env: Env) -> u32 {
        env.storage().instance().get(&symbol_short!("count")).unwrap_or(0)
    }
}
"#;
        let file = parse_file(src)?;
        let hits = NoParamNoAuthCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }
}
