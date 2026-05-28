//! Detects `balance == 0` guards before destructive operations instead of `balance <= 0`.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{BinOp, Expr, ExprBinary, File};

const CHECK_NAME: &str = "balance-negative-check";

/// Flags `balance == 0` (or `amount == 0`) comparisons in `#[contractimpl]` functions
/// where a subtraction or burn follows. The correct guard is `<= 0` or `>= amount`.
pub struct BalanceNegativeCheck;

impl Check for BalanceNegativeCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        for method in contractimpl_functions(file) {
            let fn_name = method.sig.ident.to_string();
            let mut v = EqZeroVisitor {
                fn_name: fn_name.clone(),
                out: &mut out,
            };
            v.visit_block(&method.block);
        }
        out
    }
}

struct EqZeroVisitor<'a> {
    fn_name: String,
    out: &'a mut Vec<Finding>,
}

impl Visit<'_> for EqZeroVisitor<'_> {
    fn visit_expr_binary(&mut self, i: &ExprBinary) {
        if let BinOp::Eq(_) = &i.op {
            if is_balance_or_amount(&i.left) && is_zero(&i.right)
                || is_balance_or_amount(&i.right) && is_zero(&i.left)
            {
                self.out.push(Finding {
                    check_name: CHECK_NAME.to_string(),
                    severity: Severity::Medium,
                    file_path: String::new(),
                    line: i.span().start().line,
                    function_name: self.fn_name.clone(),
                    description: format!(
                        "`== 0` guard on balance/amount in `{}`. Use `<= 0` to also catch \
                         unexpected negative values that could bypass the check.",
                        self.fn_name
                    ),
                });
            }
        }
        visit::visit_expr_binary(self, i);
    }
}

fn is_balance_or_amount(expr: &Expr) -> bool {
    let name = match expr {
        Expr::Path(p) => p
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default(),
        _ => return false,
    };
    let lower = name.to_lowercase();
    lower.contains("balance") || lower.contains("amount")
}

fn is_zero(expr: &Expr) -> bool {
    match expr {
        Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Int(i) => i.base10_digits() == "0",
            _ => false,
        },
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
        BalanceNegativeCheck.run(&file, src)
    }

    #[test]
    fn flags_balance_eq_zero() {
        let hits = run(r#"
use soroban_sdk::{contractimpl, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn burn(env: Env, balance: i128, amount: i128) {
        if balance == 0 { panic!("no balance"); }
        let _ = balance - amount;
    }
}
"#);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].check_name, CHECK_NAME);
        assert_eq!(hits[0].severity, Severity::Medium);
    }

    #[test]
    fn flags_amount_eq_zero() {
        let hits = run(r#"
use soroban_sdk::{contractimpl, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn transfer(env: Env, amount: i128) {
        if amount == 0 { return; }
    }
}
"#);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn does_not_flag_le_zero() {
        let hits = run(r#"
use soroban_sdk::{contractimpl, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn burn(env: Env, balance: i128, amount: i128) {
        if balance <= 0 { panic!("no balance"); }
        let _ = balance - amount;
    }
}
"#);
        assert!(hits.is_empty());
    }

    #[test]
    fn does_not_flag_unrelated_eq_zero() {
        let hits = run(r#"
use soroban_sdk::{contractimpl, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn check(env: Env, count: i128) {
        if count == 0 { return; }
    }
}
"#);
        assert!(hits.is_empty());
    }
}
