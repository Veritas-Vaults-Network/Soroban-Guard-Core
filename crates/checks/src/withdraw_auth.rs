//! `pub fn withdraw` missing `require_auth` before balance read.
//!
//! A withdraw function that reads a balance, subtracts an amount, and writes
//! the new balance back without calling `require_auth` first allows any caller
//! to drain any account's balance.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{BinOp, Block, Expr, ExprBinary, ExprMethodCall, File, Visibility};

const CHECK_NAME: &str = "withdraw-missing-auth";

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

fn is_storage_get(m: &ExprMethodCall) -> bool {
    matches!(m.method.to_string().as_str(), "get" | "get_unchecked")
        && receiver_chain_contains_storage(&m.receiver)
}

fn is_storage_set(m: &ExprMethodCall) -> bool {
    m.method == "set" && receiver_chain_contains_storage(&m.receiver)
}

fn is_require_auth(m: &ExprMethodCall) -> bool {
    matches!(
        m.method.to_string().as_str(),
        "require_auth" | "require_auth_for_args"
    )
}

#[derive(Default)]
struct WithdrawScan {
    has_require_auth: bool,
    has_storage_get: bool,
    has_subtraction: bool,
    has_storage_set: bool,
    /// True if require_auth appears before the first storage get.
    auth_before_get: bool,
    first_get_line: usize,
    get_seen: bool,
}

impl<'ast> Visit<'ast> for WithdrawScan {
    fn visit_expr_method_call(&mut self, i: &ExprMethodCall) {
        if is_require_auth(i) {
            self.has_require_auth = true;
            if !self.get_seen {
                self.auth_before_get = true;
            }
        }
        if is_storage_get(i) && !self.get_seen {
            self.has_storage_get = true;
            self.get_seen = true;
            self.first_get_line = i.span().start().line;
        }
        if is_storage_set(i) {
            self.has_storage_set = true;
        }
        visit::visit_expr_method_call(self, i);
    }

    fn visit_expr_binary(&mut self, i: &ExprBinary) {
        if matches!(i.op, BinOp::Sub(_) | BinOp::SubAssign(_)) {
            self.has_subtraction = true;
        }
        visit::visit_expr_binary(self, i);
    }
}

fn scan_block(block: &Block) -> WithdrawScan {
    let mut s = WithdrawScan::default();
    s.visit_block(block);
    s
}

pub struct WithdrawAuthCheck;

impl Check for WithdrawAuthCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        for method in contractimpl_functions(file) {
            if !matches!(method.vis, Visibility::Public(_)) {
                continue;
            }
            let name = method.sig.ident.to_string();
            if name != "withdraw" {
                continue;
            }
            let scan = scan_block(&method.block);
            // Must have the balance-read → subtract → write pattern.
            if !(scan.has_storage_get && scan.has_subtraction && scan.has_storage_set) {
                continue;
            }
            // Flag if auth is absent OR appears after the first storage read.
            if scan.has_require_auth && scan.auth_before_get {
                continue;
            }
            let line = if scan.first_get_line > 0 {
                scan.first_get_line
            } else {
                method.sig.fn_token.span().start().line
            };
            out.push(Finding {
                check_name: CHECK_NAME.to_string(),
                severity: Severity::High,
                file_path: String::new(),
                line,
                function_name: name.clone(),
                description: format!(
                    "Method `{name}` reads a balance from storage, subtracts an amount, and \
                     writes the new balance back, but does not call `require_auth()` before \
                     the balance read. Any caller can drain any account's balance. Call \
                     `require_auth()` on the account owner before reading the balance."
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
    fn flags_withdraw_without_auth() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, symbol_short, Address, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn withdraw(env: Env, from: Address, amount: i128) {
        let balance: i128 = env.storage().persistent().get(&from).unwrap();
        let new_balance = balance - amount;
        env.storage().persistent().set(&from, &new_balance);
    }
}
"#;
        let file = parse_file(src)?;
        let hits = WithdrawAuthCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].severity, Severity::High);
        assert!(hits[0].description.contains("require_auth"));
        Ok(())
    }

    #[test]
    fn no_finding_when_auth_before_read() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, Address, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn withdraw(env: Env, from: Address, amount: i128) {
        from.require_auth();
        let balance: i128 = env.storage().persistent().get(&from).unwrap();
        let new_balance = balance - amount;
        env.storage().persistent().set(&from, &new_balance);
    }
}
"#;
        let file = parse_file(src)?;
        let hits = WithdrawAuthCheck.run(&file, "");
        assert!(hits.is_empty(), "{hits:?}");
        Ok(())
    }

    #[test]
    fn no_finding_for_non_withdraw_fn() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, Address, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn transfer(env: Env, from: Address, amount: i128) {
        let balance: i128 = env.storage().persistent().get(&from).unwrap();
        let new_balance = balance - amount;
        env.storage().persistent().set(&from, &new_balance);
    }
}
"#;
        let file = parse_file(src)?;
        let hits = WithdrawAuthCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }

    #[test]
    fn no_finding_when_no_subtraction() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, Address, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn withdraw(env: Env, from: Address) {
        let _balance: i128 = env.storage().persistent().get(&from).unwrap();
        env.storage().persistent().set(&from, &0i128);
    }
}
"#;
        let file = parse_file(src)?;
        let hits = WithdrawAuthCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }

    #[test]
    fn flags_auth_after_read() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, Address, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn withdraw(env: Env, from: Address, amount: i128) {
        let balance: i128 = env.storage().persistent().get(&from).unwrap();
        from.require_auth();
        let new_balance = balance - amount;
        env.storage().persistent().set(&from, &new_balance);
    }
}
"#;
        let file = parse_file(src)?;
        let hits = WithdrawAuthCheck.run(&file, "");
        assert_eq!(hits.len(), 1, "auth after read should still be flagged");
        Ok(())
    }
}
