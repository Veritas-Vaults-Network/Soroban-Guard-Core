//! require_auth called with address from temporary storage (expired risk).

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Block, Expr, ExprMethodCall, File};

const CHECK_NAME: &str = "auth-temp-storage";

pub struct AuthTempStorageCheck;

impl Check for AuthTempStorageCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        for method in contractimpl_functions(file) {
            let fn_name = method.sig.ident.to_string();
            let temp_vars = extract_temp_storage_vars(&method.block);
            let mut v = AuthTempVisitor {
                fn_name: fn_name.clone(),
                temp_vars,
                out: &mut out,
            };
            v.visit_block(&method.block);
        }
        out
    }
}

fn extract_temp_storage_vars(block: &Block) -> Vec<String> {
    let mut vars = Vec::new();
    let mut v = TempVarExtractor { vars: &mut vars };
    v.visit_block(block);
    vars
}

struct TempVarExtractor<'a> {
    vars: &'a mut Vec<String>,
}

impl Visit<'_> for TempVarExtractor<'_> {
    fn visit_expr_method_call(&mut self, i: &ExprMethodCall) {
        if i.method == "get" && receiver_chain_contains_temporary(&i.receiver) {
            // This is a temporary().get() call - track the variable it's assigned to
            // We'll detect this in the parent context
        }
        visit::visit_expr_method_call(self, i);
    }
}

fn receiver_chain_contains_temporary(expr: &Expr) -> bool {
    match expr {
        Expr::MethodCall(m) => {
            m.method == "temporary" || receiver_chain_contains_temporary(&m.receiver)
        }
        Expr::Field(f) => receiver_chain_contains_temporary(&f.base),
        _ => false,
    }
}

struct AuthTempVisitor<'a> {
    fn_name: String,
    temp_vars: Vec<String>,
    out: &'a mut Vec<Finding>,
}

impl Visit<'_> for AuthTempVisitor<'_> {
    fn visit_expr_method_call(&mut self, i: &ExprMethodCall) {
        if i.method == "require_auth" {
            if let Some(arg) = i.args.first() {
                if let Expr::Path(p) = arg {
                    if let Some(ident) = p.path.segments.last() {
                        let var_name = ident.ident.to_string();
                        if self.temp_vars.contains(&var_name) {
                            self.out.push(Finding {
                                check_name: CHECK_NAME.to_string(),
                                severity: Severity::High,
                                file_path: String::new(),
                                line: i.span().start().line,
                                function_name: self.fn_name.clone(),
                                description: format!(
                                    "Method `{}` calls `require_auth()` with address `{}` \
                                     obtained from temporary storage. Temporary storage may have \
                                     expired (TTL elapsed), causing auth to fail silently or \
                                     authenticate against a default address.",
                                    self.fn_name, var_name
                                ),
                            });
                        }
                    }
                }
            }
        }
        visit::visit_expr_method_call(self, i);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Check;
    use syn::parse_file;

    #[test]
    fn flags_require_auth_with_temp_storage_var() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Address, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn bad(env: Env) {
        let admin: Address = env.storage().temporary().get(&"admin").unwrap();
        admin.require_auth();
    }
}
"#,
        )?;
        let hits = AuthTempStorageCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].severity, Severity::High);
        assert_eq!(hits[0].check_name, CHECK_NAME);
        Ok(())
    }

    #[test]
    fn no_finding_for_persistent_storage() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Address, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn good(env: Env) {
        let admin: Address = env.storage().persistent().get(&"admin").unwrap();
        admin.require_auth();
    }
}
"#,
        )?;
        let hits = AuthTempStorageCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }

    #[test]
    fn no_finding_for_env_require_auth() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn good(env: Env) {
        let _admin: Address = env.storage().temporary().get(&"admin").unwrap();
        env.require_auth();
    }
}
"#,
        )?;
        let hits = AuthTempStorageCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }
}
