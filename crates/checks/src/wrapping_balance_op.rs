//! Wrapping arithmetic on token balance (silent overflow).
//!
//! Explicitly using wrapping_add or wrapping_sub on a variable used as a token balance
//! bypasses Rust's overflow checks and can silently wrap balances to incorrect values,
//! enabling theft or minting of tokens.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ExprMethodCall, File, Stmt};

const CHECK_NAME: &str = "wrapping-balance-op";

/// Flags wrapping_add / wrapping_sub / wrapping_mul method calls on variables that are
/// subsequently stored via storage().*.set() or passed to a token client.
pub struct WrappingBalanceOpCheck;

impl Check for WrappingBalanceOpCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        for method in contractimpl_functions(file) {
            let fn_name = method.sig.ident.to_string();

            // Check if function has storage writes or token operations
            let has_storage_write = method.block.stmts.iter().any(stmt_has_storage_set);
            let has_token_op = method.block.stmts.iter().any(stmt_has_token_call);

            if has_storage_write || has_token_op {
                let mut v = WrappingOpVisitor {
                    fn_name: fn_name.clone(),
                    out: &mut out,
                };
                v.visit_block(&method.block);
            }
        }
        out
    }
}

struct WrappingOpVisitor<'a> {
    fn_name: String,
    out: &'a mut Vec<Finding>,
}

impl Visit<'_> for WrappingOpVisitor<'_> {
    fn visit_expr_method_call(&mut self, i: &ExprMethodCall) {
        let method_name = i.method.to_string();
        if matches!(
            method_name.as_str(),
            "wrapping_add" | "wrapping_sub" | "wrapping_mul"
        ) {
            self.out.push(Finding {
                check_name: CHECK_NAME.to_string(),
                severity: Severity::High,
                file_path: String::new(),
                line: i.span().start().line,
                function_name: self.fn_name.clone(),
                description: format!(
                    "Function `{}` uses `{}` on a value that flows into storage or token operations. \
                     Wrapping arithmetic bypasses overflow checks and can silently produce incorrect \
                     balances, enabling theft or unauthorized minting. Use checked arithmetic instead.",
                    self.fn_name, method_name
                ),
            });
        }
        visit::visit_expr_method_call(self, i);
    }
}

fn method_chain_contains(expr: &Expr, name: &str) -> bool {
    match expr {
        Expr::MethodCall(m) => {
            if m.method == name {
                return true;
            }
            method_chain_contains(&m.receiver, name)
        }
        _ => false,
    }
}

fn expr_has_storage_set(expr: &Expr) -> bool {
    match expr {
        Expr::MethodCall(m) => {
            if m.method == "set" && method_chain_contains(&m.receiver, "storage") {
                return true;
            }
            expr_has_storage_set(&m.receiver)
        }
        _ => false,
    }
}

fn expr_has_token_call(expr: &Expr) -> bool {
    match expr {
        Expr::MethodCall(m) => {
            let method_name = m.method.to_string();
            if matches!(
                method_name.as_str(),
                "transfer" | "mint" | "burn" | "approve"
            ) {
                return true;
            }
            expr_has_token_call(&m.receiver)
        }
        _ => false,
    }
}

fn stmt_has_storage_set(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Expr(e, _) => expr_has_storage_set(e),
        _ => false,
    }
}

fn stmt_has_token_call(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Expr(e, _) => expr_has_token_call(e),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_file;

    #[test]
    fn detects_wrapping_add_with_storage() {
        let code = r#"
use soroban_sdk::{contract, contractimpl, Env, Symbol};

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn deposit(env: Env, amount: i128) {
        let balance: i128 = env.storage().persistent().get(&Symbol::new(&env, "bal")).unwrap_or(0);
        let new_balance = balance.wrapping_add(amount);
        env.storage().persistent().set(&Symbol::new(&env, "bal"), &new_balance);
    }
}
        "#;
        let file = parse_file(code).unwrap();
        let check = WrappingBalanceOpCheck;
        let findings = check.run(&file, code);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::High);
        assert_eq!(findings[0].check_name, CHECK_NAME);
    }

    #[test]
    fn detects_wrapping_sub_with_storage() {
        let code = r#"
use soroban_sdk::{contract, contractimpl, Env, Symbol};

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn withdraw(env: Env, amount: i128) {
        let balance: i128 = env.storage().persistent().get(&Symbol::new(&env, "bal")).unwrap_or(0);
        let new_balance = balance.wrapping_sub(amount);
        env.storage().persistent().set(&Symbol::new(&env, "bal"), &new_balance);
    }
}
        "#;
        let file = parse_file(code).unwrap();
        let check = WrappingBalanceOpCheck;
        let findings = check.run(&file, code);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::High);
    }

    #[test]
    fn allows_checked_add() {
        let code = r#"
use soroban_sdk::{contract, contractimpl, Env, Symbol};

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn deposit(env: Env, amount: i128) {
        let balance: i128 = env.storage().persistent().get(&Symbol::new(&env, "bal")).unwrap_or(0);
        let new_balance = balance.checked_add(amount).expect("overflow");
        env.storage().persistent().set(&Symbol::new(&env, "bal"), &new_balance);
    }
}
        "#;
        let file = parse_file(code).unwrap();
        let check = WrappingBalanceOpCheck;
        let findings = check.run(&file, code);
        assert!(findings.is_empty());
    }

    #[test]
    fn detects_wrapping_mul_with_token() {
        let code = r#"
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn multiply_and_transfer(env: Env, token: Address, to: Address, amount: i128, multiplier: i128) {
        let total = amount.wrapping_mul(multiplier);
        token.transfer(&env, &to, &total);
    }
}
        "#;
        let file = parse_file(code).unwrap();
        let check = WrappingBalanceOpCheck;
        let findings = check.run(&file, code);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::High);
    }
}
