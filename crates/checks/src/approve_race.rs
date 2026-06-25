//! Detects approve methods that overwrite allowances without a zero reset or guard.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Block, Expr, ExprIf, ExprMacro, ExprMethodCall, File, Lit, Macro};

const CHECK_NAME: &str = "approve-race-condition";

pub struct ApproveRaceCheck;

impl Check for ApproveRaceCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        for method in contractimpl_functions(file) {
            let fn_name = method.sig.ident.to_string();
            if !is_approve_like(&fn_name) {
                continue;
            }

            let mut scan = ApproveBodyScan::default();
            scan.visit_block(&method.block);
            if !scan.has_non_zero_allowance_set || scan.has_zero_reset || scan.has_guard {
                continue;
            }

            let line = first_non_zero_allowance_set_line(&method.block)
                .unwrap_or_else(|| method.sig.ident.span().start().line);
            out.push(Finding {
                check_name: CHECK_NAME.to_string(),
                severity: Severity::High,
                file_path: String::new(),
                line,
                function_name: fn_name.clone(),
                description: format!(
                    "Approve-like method `{fn_name}` writes a non-zero allowance without \
                     resetting the allowance to zero first or checking the existing value. \
                     A spender may front-run the change and spend both the old and new allowance."
                ),
            });
        }
        out
    }
}

fn is_approve_like(name: &str) -> bool {
    name.to_ascii_lowercase().contains("approve")
}

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

fn is_storage_set_call(m: &ExprMethodCall) -> bool {
    m.method == "set" && receiver_chain_contains_storage(&m.receiver)
}

fn expr_is_zero_literal(expr: &Expr) -> bool {
    match expr {
        Expr::Reference(r) => expr_is_zero_literal(&r.expr),
        Expr::Lit(l) => matches!(&l.lit, Lit::Int(i) if i.base10_digits() == "0"),
        _ => false,
    }
}

fn set_call_has_zero_arg(m: &ExprMethodCall) -> bool {
    m.args.iter().any(expr_is_zero_literal)
}

#[derive(Default)]
struct ApproveBodyScan {
    has_non_zero_allowance_set: bool,
    has_zero_reset: bool,
    has_guard: bool,
}

impl<'ast> Visit<'ast> for ApproveBodyScan {
    fn visit_expr_method_call(&mut self, i: &'ast ExprMethodCall) {
        if is_storage_set_call(i) {
            if set_call_has_zero_arg(i) {
                self.has_zero_reset = true;
            } else {
                self.has_non_zero_allowance_set = true;
            }
        }
        visit::visit_expr_method_call(self, i);
    }

    fn visit_expr_macro(&mut self, i: &'ast ExprMacro) {
        if i.mac.path.is_ident("assert") {
            self.has_guard = true;
        }
        visit::visit_expr_macro(self, i);
    }

    fn visit_macro(&mut self, i: &'ast Macro) {
        if i.path.is_ident("assert") {
            self.has_guard = true;
        }
        visit::visit_macro(self, i);
    }

    fn visit_expr_if(&mut self, i: &'ast ExprIf) {
        self.has_guard = true;
        visit::visit_expr_if(self, i);
    }
}

struct FirstNonZeroAllowanceSet {
    line: Option<usize>,
}

impl<'ast> Visit<'ast> for FirstNonZeroAllowanceSet {
    fn visit_expr_method_call(&mut self, i: &'ast ExprMethodCall) {
        if self.line.is_none() && is_storage_set_call(i) && !set_call_has_zero_arg(i) {
            self.line = Some(i.span().start().line);
        }
        visit::visit_expr_method_call(self, i);
    }
}

fn first_non_zero_allowance_set_line(block: &Block) -> Option<usize> {
    let mut visitor = FirstNonZeroAllowanceSet { line: None };
    visitor.visit_block(block);
    visitor.line
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Check, Severity};
    use syn::parse_file;

    fn run(src: &str) -> Result<Vec<Finding>, syn::Error> {
        let file = parse_file(src)?;
        Ok(ApproveRaceCheck.run(&file, src))
    }

    #[test]
    fn flags_approve_storage_set_without_zero_reset_or_guard() -> Result<(), syn::Error> {
        let hits = run(r#"
use soroban_sdk::{contractimpl, Address, Env};

pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn approve(env: Env, owner: Address, spender: Address, amount: i128) {
        let allowance_key = (owner, spender);
        env.storage().persistent().set(&allowance_key, &amount);
    }
}
"#)?;

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].check_name, CHECK_NAME);
        assert_eq!(hits[0].severity, Severity::High);
        assert_eq!(hits[0].function_name, "approve");
        Ok(())
    }

    #[test]
    fn passes_when_approve_resets_allowance_to_zero_first() -> Result<(), syn::Error> {
        let hits = run(r#"
use soroban_sdk::{contractimpl, Address, Env};

pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn approve(env: Env, owner: Address, spender: Address, amount: i128) {
        let allowance_key = (owner, spender);
        env.storage().persistent().set(&allowance_key, &0);
        env.storage().persistent().set(&allowance_key, &amount);
    }
}
"#)?;

        assert!(hits.is_empty());
        Ok(())
    }

    #[test]
    fn passes_when_approve_has_assert_guard() -> Result<(), syn::Error> {
        let hits = run(r#"
use soroban_sdk::{contractimpl, Address, Env};

pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn approve(env: Env, owner: Address, spender: Address, amount: i128) {
        assert!(amount >= 0);
        let allowance_key = (owner, spender);
        env.storage().persistent().set(&allowance_key, &amount);
    }
}
"#)?;

        assert!(hits.is_empty());
        Ok(())
    }

    #[test]
    fn passes_when_approve_has_if_guard() -> Result<(), syn::Error> {
        let hits = run(r#"
use soroban_sdk::{contractimpl, Address, Env};

pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn approve(env: Env, owner: Address, spender: Address, amount: i128) {
        if amount < 0 {
            return;
        }
        let allowance_key = (owner, spender);
        env.storage().persistent().set(&allowance_key, &amount);
    }
}
"#)?;

        assert!(hits.is_empty());
        Ok(())
    }

    #[test]
    fn ignores_non_approve_storage_set() -> Result<(), syn::Error> {
        let hits = run(r#"
use soroban_sdk::{contractimpl, Address, Env};

pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn set_allowance(env: Env, owner: Address, spender: Address, amount: i128) {
        let allowance_key = (owner, spender);
        env.storage().persistent().set(&allowance_key, &amount);
    }
}
"#)?;

        assert!(hits.is_empty());
        Ok(())
    }
}
