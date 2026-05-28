//! Detects token allowance set to i128::MAX / u128::MAX (unlimited approval).
//!
//! Setting an allowance to the maximum integer value is an unlimited-approval
//! anti-pattern. If the spender is compromised or the contract is buggy, all
//! funds can be drained.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::visit::{self, Visit};
use syn::{Expr, ExprMethodCall, File, Lit};

const CHECK_NAME: &str = "unlimited-allowance";

/// i128::MAX = 170141183460469231731687303715884105727
const I128_MAX: u128 = 170_141_183_460_469_231_731_687_303_715_884_105_727;
/// u128::MAX = 340282366920938463463374607431768211455
const U128_MAX: u128 = 340_282_366_920_938_463_463_374_607_431_768_211_455;

pub struct UnlimitedAllowanceCheck;

impl Check for UnlimitedAllowanceCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        for method in contractimpl_functions(file) {
            let fn_name = method.sig.ident.to_string();
            let mut scan = ApproveScan::default();
            scan.visit_block(&method.block);
            for line in scan.findings {
                out.push(Finding {
                    check_name: CHECK_NAME.to_string(),
                    severity: Severity::Medium,
                    file_path: String::new(),
                    line,
                    function_name: fn_name.clone(),
                    description: "Call to `approve` with an amount equal to `i128::MAX` or \
                                  `u128::MAX` grants unlimited token approval. If the spender \
                                  is compromised, all funds can be drained. Use a bounded \
                                  allowance instead."
                        .to_string(),
                });
            }
        }
        out
    }
}

fn is_max_literal(expr: &Expr) -> bool {
    match expr {
        // &i128::MAX — unwrap the reference
        Expr::Reference(r) => is_max_literal(&r.expr),
        // Plain integer literal: 170141183460469231731687303715884105727
        Expr::Lit(l) => {
            if let Lit::Int(i) = &l.lit {
                if let Ok(v) = i.base10_parse::<u128>() {
                    return v == I128_MAX || v == U128_MAX;
                }
            }
            false
        }
        // i128::MAX or u128::MAX path expression
        Expr::Path(p) => {
            let segs: Vec<_> = p.path.segments.iter().collect();
            if segs.len() == 2 {
                let ty = segs[0].ident.to_string();
                let field = segs[1].ident.to_string();
                return field == "MAX" && matches!(ty.as_str(), "i128" | "u128");
            }
            false
        }
        // Handle cast just in case
        Expr::Cast(c) => is_max_literal(&c.expr),
        _ => false,
    }
}

#[derive(Default)]
struct ApproveScan {
    findings: Vec<usize>,
}

impl<'ast> Visit<'ast> for ApproveScan {
    fn visit_expr_method_call(&mut self, i: &'ast ExprMethodCall) {
        if i.method == "approve" {
            // approve(from, spender, amount, expiration_ledger)
            // The amount is the 3rd argument (index 2).
            if let Some(amount_arg) = i.args.iter().nth(2) {
                if is_max_literal(amount_arg) {
                    self.findings.push(i.method.span().start().line);
                }
            }
        }
        visit::visit_expr_method_call(self, i);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_file;

    #[test]
    fn flags_i128_max_path() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env, Address};
pub struct C;
#[contractimpl]
impl C {
    pub fn approve_all(env: Env, from: Address, spender: Address) {
        let token = token::Client::new(&env, &token_id);
        token.approve(&from, &spender, &i128::MAX, &999999);
    }
}
"#,
        )?;
        let hits = UnlimitedAllowanceCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].severity, Severity::Medium);
        Ok(())
    }

    #[test]
    fn flags_u128_max_path() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env, Address};
pub struct C;
#[contractimpl]
impl C {
    pub fn approve_all(env: Env, from: Address, spender: Address) {
        let token = token::Client::new(&env, &token_id);
        token.approve(&from, &spender, &u128::MAX, &999999);
    }
}
"#,
        )?;
        let hits = UnlimitedAllowanceCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].severity, Severity::Medium);
        Ok(())
    }

    #[test]
    fn flags_i128_max_literal() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env, Address};
pub struct C;
#[contractimpl]
impl C {
    pub fn approve_all(env: Env, from: Address, spender: Address) {
        let token = token::Client::new(&env, &token_id);
        token.approve(&from, &spender, &170141183460469231731687303715884105727_i128, &999999);
    }
}
"#,
        )?;
        let hits = UnlimitedAllowanceCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        Ok(())
    }

    #[test]
    fn passes_bounded_allowance() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env, Address};
pub struct C;
#[contractimpl]
impl C {
    pub fn approve_bounded(env: Env, from: Address, spender: Address, amount: i128) {
        let token = token::Client::new(&env, &token_id);
        token.approve(&from, &spender, &amount, &999999);
    }
}
"#,
        )?;
        let hits = UnlimitedAllowanceCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }

    #[test]
    fn passes_no_approve_call() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env, Address};
pub struct C;
#[contractimpl]
impl C {
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        let token = token::Client::new(&env, &token_id);
        token.transfer(&from, &to, &amount);
    }
}
"#,
        )?;
        let hits = UnlimitedAllowanceCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }
}
