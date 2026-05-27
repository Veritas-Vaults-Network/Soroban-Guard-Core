//! authorize_as_current_contract called with empty invocation vector.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ExprArray, ExprMethodCall, File};

const CHECK_NAME: &str = "authorize-empty";

pub struct AuthorizeEmptyCheck;

impl Check for AuthorizeEmptyCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        for method in contractimpl_functions(file) {
            let fn_name = method.sig.ident.to_string();
            let mut v = AuthorizeEmptyVisitor {
                fn_name: fn_name.clone(),
                out: &mut out,
            };
            v.visit_block(&method.block);
        }
        out
    }
}

struct AuthorizeEmptyVisitor<'a> {
    fn_name: String,
    out: &'a mut Vec<Finding>,
}

impl Visit<'_> for AuthorizeEmptyVisitor<'_> {
    fn visit_expr_method_call(&mut self, i: &ExprMethodCall) {
        if i.method == "authorize_as_current_contract" {
            if let Some(arg) = i.args.first() {
                if is_empty_array(arg) {
                    self.out.push(Finding {
                        check_name: CHECK_NAME.to_string(),
                        severity: Severity::High,
                        file_path: String::new(),
                        line: i.span().start().line,
                        function_name: self.fn_name.clone(),
                        description: format!(
                            "Method `{}` calls `authorize_as_current_contract(&[])` with an empty \
                             invocation vector. This authorizes nothing but still consumes compute. \
                             Provide specific sub-contract invocations to authorize.",
                            self.fn_name
                        ),
                    });
                }
            }
        }
        visit::visit_expr_method_call(self, i);
    }
}

fn is_empty_array(expr: &Expr) -> bool {
    match expr {
        Expr::Array(ExprArray { elems, .. }) => elems.is_empty(),
        Expr::Reference(r) => is_empty_array(&r.expr),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Check;
    use syn::parse_file;

    #[test]
    fn flags_empty_authorize_array() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn bad(env: Env) {
        env.authorize_as_current_contract(&[]);
    }
}
"#,
        )?;
        let hits = AuthorizeEmptyCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].severity, Severity::High);
        assert_eq!(hits[0].check_name, CHECK_NAME);
        Ok(())
    }

    #[test]
    fn no_finding_for_non_empty_array() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env, InvokeContractArgs};
pub struct C;
#[contractimpl]
impl C {
    pub fn good(env: Env, args: InvokeContractArgs) {
        env.authorize_as_current_contract(&[args]);
    }
}
"#,
        )?;
        let hits = AuthorizeEmptyCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }

    #[test]
    fn no_finding_for_vec_macro() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn good(env: Env) {
        env.authorize_as_current_contract(&vec![]);
    }
}
"#,
        )?;
        let hits = AuthorizeEmptyCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }
}
