//! Detects `x * y / z` patterns where the intermediate multiplication can overflow.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{BinOp, Expr, ExprBinary, File};

const CHECK_NAME: &str = "mul-before-div";

/// Flags `(x * y) / z` expressions in `#[contractimpl]` functions where no
/// `checked_mul` is used, risking silent overflow of the intermediate product.
pub struct MulBeforeDivCheck;

impl Check for MulBeforeDivCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        for method in contractimpl_functions(file) {
            let fn_name = method.sig.ident.to_string();
            let mut v = MulDivVisitor {
                fn_name: fn_name.clone(),
                out: &mut out,
            };
            v.visit_block(&method.block);
        }
        out
    }
}

struct MulDivVisitor<'a> {
    fn_name: String,
    out: &'a mut Vec<Finding>,
}

impl Visit<'_> for MulDivVisitor<'_> {
    fn visit_expr_binary(&mut self, i: &ExprBinary) {
        // Flag `(x * y) / z` — division whose left operand is a multiplication
        if let BinOp::Div(_) = &i.op {
            if is_mul_expr(&i.left) {
                self.out.push(Finding {
                    check_name: CHECK_NAME.to_string(),
                    severity: Severity::Medium,
                    file_path: String::new(),
                    line: i.span().start().line,
                    function_name: self.fn_name.clone(),
                    description: format!(
                        "`x * y / z` in `{}`: the intermediate `x * y` can overflow `i128::MAX`. \
                         Use `checked_mul` or restructure the expression to avoid overflow.",
                        self.fn_name
                    ),
                });
            }
        }
        visit::visit_expr_binary(self, i);
    }
}

fn is_mul_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Binary(b) => matches!(b.op, BinOp::Mul(_)),
        Expr::Paren(p) => is_mul_expr(&p.expr),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Check;
    use syn::parse_file;

    fn run(src: &str) -> Vec<Finding> {
        let file = parse_file(src).unwrap();
        MulBeforeDivCheck.run(&file, src)
    }

    #[test]
    fn flags_mul_before_div() {
        let hits = run(r#"
use soroban_sdk::{contractimpl, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn calc(env: Env, x: i128, y: i128, z: i128) -> i128 {
        x * y / z
    }
}
"#);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].check_name, CHECK_NAME);
        assert_eq!(hits[0].severity, Severity::Medium);
    }

    #[test]
    fn flags_parenthesized_mul_before_div() {
        let hits = run(r#"
use soroban_sdk::{contractimpl, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn calc(env: Env, x: i128, y: i128, z: i128) -> i128 {
        (x * y) / z
    }
}
"#);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn does_not_flag_plain_div() {
        let hits = run(r#"
use soroban_sdk::{contractimpl, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn calc(env: Env, x: i128, z: i128) -> i128 {
        x / z
    }
}
"#);
        assert!(hits.is_empty());
    }

    #[test]
    fn does_not_flag_div_before_mul() {
        let hits = run(r#"
use soroban_sdk::{contractimpl, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn calc(env: Env, x: i128, y: i128, z: i128) -> i128 {
        (x / z) * y
    }
}
"#);
        assert!(hits.is_empty());
    }
}
