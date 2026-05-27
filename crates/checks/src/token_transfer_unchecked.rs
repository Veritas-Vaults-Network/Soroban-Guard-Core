//! Flags `soroban_sdk::token::Client::transfer(...)` calls whose return value is ignored.
//!
//! `token::Client::transfer` can panic if the sender has insufficient balance.
//! When the call result is used as a statement expression (not bound to a variable
//! or matched), the contract proceeds as if the transfer succeeded, leading to
//! silent accounting errors.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, File, Stmt};

const CHECK_NAME: &str = "token-transfer-unchecked";

/// Flags `.transfer(...)` method calls inside `#[contractimpl]` functions where
/// the result is used as a statement expression (not bound to a variable or matched).
pub struct TokenTransferUncheckedCheck;

impl Check for TokenTransferUncheckedCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        for method in contractimpl_functions(file) {
            let fn_name = method.sig.ident.to_string();
            let mut v = TransferVisitor {
                fn_name: fn_name.clone(),
                out: &mut out,
            };
            v.visit_block(&method.block);
        }
        out
    }
}

/// Returns true if the receiver is NOT a bare `env` path.
fn receiver_is_not_bare_env(expr: &Expr) -> bool {
    match expr {
        Expr::Path(p) => !p.path.is_ident("env"),
        _ => true,
    }
}

struct TransferVisitor<'a> {
    fn_name: String,
    out: &'a mut Vec<Finding>,
}

impl<'ast> Visit<'ast> for TransferVisitor<'ast> {
    fn visit_stmt(&mut self, i: &'ast Stmt) {
        if let Stmt::Expr(Expr::MethodCall(m), Some(_semi)) = i {
            if m.method == "transfer" && receiver_is_not_bare_env(&m.receiver) {
                self.out.push(Finding {
                    check_name: CHECK_NAME.to_string(),
                    severity: Severity::Medium,
                    file_path: String::new(),
                    line: m.span().start().line,
                    function_name: self.fn_name.clone(),
                    description: format!(
                        "Method `{}` calls `.transfer(...)` but ignores the return value. \
                         `soroban_sdk::token::Client::transfer` can fail if the sender has \
                         insufficient balance; discarding the result means the contract \
                         proceeds as if the transfer succeeded, causing accounting errors.",
                        self.fn_name
                    ),
                });
            }
        }
        visit::visit_stmt(self, i);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Check;
    use syn::parse_file;

    fn run(src: &str) -> Vec<Finding> {
        TokenTransferUncheckedCheck.run(&parse_file(src).unwrap(), src)
    }

    #[test]
    fn flags_transfer_as_bare_statement() {
        let hits = run(r#"
pub struct C;
#[contractimpl]
impl C {
    pub fn pay(env: Env, token: Address, from: Address, to: Address, amount: i128) {
        let client = token::Client::new(&env, &token);
        client.transfer(&from, &to, &amount);
    }
}
"#);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].check_name, CHECK_NAME);
        assert_eq!(hits[0].severity, Severity::Medium);
        assert_eq!(hits[0].function_name, "pay");
    }

    #[test]
    fn flags_inline_client_transfer_as_statement() {
        let hits = run(r#"
pub struct C;
#[contractimpl]
impl C {
    pub fn pay(env: Env, token: Address, from: Address, to: Address, amount: i128) {
        token::Client::new(&env, &token).transfer(&from, &to, &amount);
    }
}
"#);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].function_name, "pay");
    }

    #[test]
    fn flags_multiple_unchecked_transfers_in_one_fn() {
        let hits = run(r#"
pub struct C;
#[contractimpl]
impl C {
    pub fn batch(env: Env, t: Address, a: Address, b: Address, c: Address, amt: i128) {
        let client = token::Client::new(&env, &t);
        client.transfer(&a, &b, &amt);
        client.transfer(&b, &c, &amt);
    }
}
"#);
        assert_eq!(hits.len(), 2);
    }

    #[test]
    fn passes_when_result_bound_to_variable() {
        let hits = run(r#"
pub struct C;
#[contractimpl]
impl C {
    pub fn pay(env: Env, token: Address, from: Address, to: Address, amount: i128) {
        let client = token::Client::new(&env, &token);
        let _result = client.transfer(&from, &to, &amount);
    }
}
"#);
        assert!(hits.is_empty());
    }

    #[test]
    fn passes_when_result_used_in_return_position() {
        let hits = run(r#"
pub struct C;
#[contractimpl]
impl C {
    pub fn pay(env: Env, token: Address, from: Address, to: Address, amount: i128) -> bool {
        let client = token::Client::new(&env, &token);
        let ok = client.transfer(&from, &to, &amount);
        ok
    }
}
"#);
        assert!(hits.is_empty());
    }

    #[test]
    fn ignores_non_contractimpl_impl() {
        let hits = run(r#"
pub struct C;
impl C {
    pub fn pay(env: Env, token: Address, from: Address, to: Address, amount: i128) {
        let client = token::Client::new(&env, &token);
        client.transfer(&from, &to, &amount);
    }
}
"#);
        assert!(hits.is_empty());
    }

    #[test]
    fn ignores_env_transfer_not_token_client() {
        let hits = run(r#"
pub struct C;
#[contractimpl]
impl C {
    pub fn pay(env: Env, to: Address, amount: i128) {
        env.transfer(&to, &amount);
    }
}
"#);
        assert!(hits.is_empty());
    }
}
