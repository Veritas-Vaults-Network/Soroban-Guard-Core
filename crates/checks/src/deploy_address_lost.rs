//! Detects `env.deployer().deploy(...)` calls where the returned deployed contract address is dropped / not saved.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ExprMethodCall, File, Pat, Stmt};

const CHECK_NAME: &str = "deploy-address-lost";

pub struct DeployAddressLostCheck;

impl Check for DeployAddressLostCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut findings = Vec::new();

        for method in contractimpl_functions(file) {
            let fn_name = method.sig.ident.to_string();
            let mut scan = DeployAddressLostScan {
                fn_name,
                findings: &mut findings,
            };
            scan.visit_block(&method.block);
        }

        findings
    }
}

/// Returns true if `call` is a `.deploy(...)` method call.
fn is_deploy_call(call: &ExprMethodCall) -> bool {
    call.method == "deploy"
}

struct DeployAddressLostScan<'a> {
    fn_name: String,
    findings: &'a mut Vec<Finding>,
}

impl<'ast> Visit<'ast> for DeployAddressLostScan<'_> {
    fn visit_stmt(&mut self, stmt: &'ast Stmt) {
        match stmt {
            // Case 1: Statement is an expression with a semicolon (return value dropped directly)
            // e.g. env.deployer().deploy(...);
            Stmt::Expr(expr, Some(_semi)) => {
                if let Expr::MethodCall(call) = expr {
                    if is_deploy_call(call) {
                        self.findings.push(Finding {
                            check_name: CHECK_NAME.to_string(),
                            severity: Severity::Medium,
                            file_path: String::new(),
                            line: call.span().start().line,
                            function_name: self.fn_name.clone(),
                            description: format!(
                                "Function `{}` deploys a contract via `.deploy()` but discards the resulting contract address. \
                                 The deployed address is not stored, returned, or emitted, making the deployed contract unrecoverable.",
                                self.fn_name
                            ),
                        });
                    }
                }
            }
            // Case 2: Statement is `let _ = deploy(...);` (explicit discard)
            Stmt::Local(local) => {
                if let Some(init) = &local.init {
                    if let Expr::MethodCall(call) = &*init.expr {
                        if is_deploy_call(call) && matches!(local.pat, Pat::Wild(_)) {
                            self.findings.push(Finding {
                                check_name: CHECK_NAME.to_string(),
                                severity: Severity::Medium,
                                file_path: String::new(),
                                line: call.span().start().line,
                                function_name: self.fn_name.clone(),
                                description: format!(
                                    "Function `{}` deploys a contract via `.deploy()` but explicitly discards the resulting address with `let _`. \
                                     The deployed address is not stored, returned, or emitted.",
                                    self.fn_name
                                ),
                            });
                        }
                    }
                }
            }
            _ => {}
        }

        visit::visit_stmt(self, stmt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_file;

    fn run(code: &str) -> Vec<Finding> {
        let file = parse_file(code).unwrap();
        DeployAddressLostCheck.run(&file, code)
    }

    #[test]
    fn flags_unbound_deploy_call() {
        let code = r#"
#[contractimpl]
impl MyContract {
    pub fn deploy_sub(env: Env) {
        env.deployer().deploy(
            &env.current_contract_wasm(),
            &[],
            &Symbol::new(&env, "init"),
            &[],
        );
    }
}
"#;
        let findings = run(code);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].check_name, CHECK_NAME);
        assert_eq!(findings[0].severity, Severity::Medium);
    }

    #[test]
    fn flags_wildcard_let_deploy_call() {
        let code = r#"
#[contractimpl]
impl MyContract {
    pub fn deploy_sub(env: Env) {
        let _ = env.deployer().deploy(
            &env.current_contract_wasm(),
            &[],
            &Symbol::new(&env, "init"),
            &[],
        );
    }
}
"#;
        let findings = run(code);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].check_name, CHECK_NAME);
    }

    #[test]
    fn passes_when_address_bound_and_stored() {
        let code = r#"
#[contractimpl]
impl MyContract {
    pub fn deploy_sub(env: Env) {
        let addr = env.deployer().deploy(
            &env.current_contract_wasm(),
            &[],
            &Symbol::new(&env, "init"),
            &[],
        );
        env.storage().instance().set(&symbol_short!("sub"), &addr);
    }
}
"#;
        let findings = run(code);
        assert!(findings.is_empty());
    }

    #[test]
    fn passes_when_address_returned() {
        let code = r#"
#[contractimpl]
impl MyContract {
    pub fn deploy_sub(env: Env) -> Address {
        env.deployer().deploy(
            &env.current_contract_wasm(),
            &[],
            &Symbol::new(&env, "init"),
            &[],
        )
    }
}
"#;
        let findings = run(code);
        assert!(findings.is_empty());
    }
}
