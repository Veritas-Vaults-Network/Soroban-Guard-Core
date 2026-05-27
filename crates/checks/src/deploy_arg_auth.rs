//! require_auth called before deployer().deploy() but deploy args not covered by auth.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Block, Expr, ExprMethodCall, File, Stmt};

const CHECK_NAME: &str = "deploy-arg-auth";

pub struct DeployArgAuthCheck;

impl Check for DeployArgAuthCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        for method in contractimpl_functions(file) {
            let fn_name = method.sig.ident.to_string();
            let param_names = extract_param_names(&method.sig.inputs);
            let mut v = DeployAuthVisitor {
                fn_name: fn_name.clone(),
                param_names,
                out: &mut out,
            };
            v.visit_block(&method.block);
        }
        out
    }
}

fn extract_param_names(inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>) -> Vec<String> {
    let mut names = Vec::new();
    for arg in inputs {
        if let syn::FnArg::Typed(pt) = arg {
            if let syn::Pat::Ident(ident) = &*pt.pat {
                names.push(ident.ident.to_string());
            }
        }
    }
    names
}

struct DeployAuthVisitor<'a> {
    fn_name: String,
    param_names: Vec<String>,
    out: &'a mut Vec<Finding>,
}

impl Visit<'_> for DeployAuthVisitor<'_> {
    fn visit_block(&mut self, i: &Block) {
        let mut has_require_auth = false;
        let mut deploy_call_line = None;

        for stmt in &i.stmts {
            if let Stmt::Expr(expr, _) | Stmt::Semi(expr, _) = stmt {
                if is_require_auth_call(expr) {
                    has_require_auth = true;
                }
                if let Some(line) = check_deploy_with_params(expr, &self.param_names) {
                    deploy_call_line = Some(line);
                }
            }
        }

        if has_require_auth && deploy_call_line.is_some() {
            if !has_require_auth_for_args(&i.stmts, &self.param_names) {
                self.out.push(Finding {
                    check_name: CHECK_NAME.to_string(),
                    severity: Severity::High,
                    file_path: String::new(),
                    line: deploy_call_line.unwrap(),
                    function_name: self.fn_name.clone(),
                    description: format!(
                        "Method `{}` calls `env.deployer().deploy()` with function parameters, \
                         but only calls `require_auth()` without binding to the deploy arguments. \
                         Use `require_auth_for_args()` to bind auth to wasm_hash and salt.",
                        self.fn_name
                    ),
                });
            }
        }

        visit::visit_block(self, i);
    }
}

fn is_require_auth_call(expr: &Expr) -> bool {
    if let Expr::MethodCall(m) = expr {
        m.method == "require_auth"
    } else {
        false
    }
}

fn check_deploy_with_params(expr: &Expr, param_names: &[String]) -> Option<usize> {
    if let Expr::MethodCall(m) = expr {
        if m.method == "deploy" {
            for arg in &m.args {
                if let Expr::Path(p) = arg {
                    if let Some(ident) = p.path.segments.last() {
                        if param_names.contains(&ident.ident.to_string()) {
                            return Some(m.span().start().line);
                        }
                    }
                }
            }
        }
        check_deploy_with_params(&m.receiver, param_names)
    } else if let Expr::Call(c) = expr {
        for arg in &c.args {
            if let Expr::Path(p) = arg {
                if let Some(ident) = p.path.segments.last() {
                    if param_names.contains(&ident.ident.to_string()) {
                        return Some(c.span().start().line);
                    }
                }
            }
        }
        None
    } else {
        None
    }
}

fn has_require_auth_for_args(stmts: &[Stmt], param_names: &[String]) -> bool {
    for stmt in stmts {
        if let Stmt::Expr(expr, _) | Stmt::Semi(expr, _) = stmt {
            if is_require_auth_for_args_with_params(expr, param_names) {
                return true;
            }
        }
    }
    false
}

fn is_require_auth_for_args_with_params(expr: &Expr, param_names: &[String]) -> bool {
    if let Expr::MethodCall(m) = expr {
        if m.method == "require_auth_for_args" {
            for arg in &m.args {
                if let Expr::Tuple(t) = arg {
                    for elem in &t.elems {
                        if let Expr::Path(p) = elem {
                            if let Some(ident) = p.path.segments.last() {
                                if param_names.contains(&ident.ident.to_string()) {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Check;
    use syn::parse_file;

    #[test]
    fn flags_deploy_with_param_and_require_auth() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env, BytesN};
pub struct C;
#[contractimpl]
impl C {
    pub fn bad(env: Env, wasm_hash: BytesN<32>, salt: u64) {
        env.require_auth();
        env.deployer().deploy(wasm_hash, salt);
    }
}
"#,
        )?;
        let hits = DeployArgAuthCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].severity, Severity::High);
        Ok(())
    }

    #[test]
    fn no_finding_with_require_auth_for_args() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env, BytesN};
pub struct C;
#[contractimpl]
impl C {
    pub fn good(env: Env, wasm_hash: BytesN<32>, salt: u64) {
        env.require_auth_for_args((wasm_hash, salt));
        env.deployer().deploy(wasm_hash, salt);
    }
}
"#,
        )?;
        let hits = DeployArgAuthCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }

    #[test]
    fn no_finding_for_literal_args() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn good(env: Env) {
        env.require_auth();
        env.deployer().deploy(&[1, 2, 3], 0u64);
    }
}
"#,
        )?;
        let hits = DeployArgAuthCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }
}
