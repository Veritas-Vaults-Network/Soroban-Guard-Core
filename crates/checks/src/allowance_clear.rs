//! Token approval not cleared after transfer_from.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ExprMethodCall, File};

const CHECK_NAME: &str = "allowance-not-cleared";

pub struct AllowanceClearCheck;

impl Check for AllowanceClearCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        for method in contractimpl_functions(file) {
            let fn_name = method.sig.ident.to_string();
            if fn_name != "transfer_from" {
                continue;
            }
            let mut scan = AllowanceScan::default();
            scan.visit_block(&method.block);
            if scan.has_storage_read && !scan.has_storage_write {
                let line = method.sig.fn_token.span().start().line;
                out.push(Finding {
                    check_name: CHECK_NAME.to_string(),
                    severity: Severity::High,
                    file_path: String::new(),
                    line,
                    function_name: fn_name,
                    description: "Method `transfer_from` reads from storage but does not write \
                                  back to clear or reduce allowance. A spender can drain the account repeatedly."
                        .to_string(),
                });
            }
        }
        out
    }
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

fn is_storage_get(m: &ExprMethodCall) -> bool {
    m.method == "get" && receiver_chain_contains_storage(&m.receiver)
}

fn is_storage_write(m: &ExprMethodCall) -> bool {
    let name = m.method.to_string();
    matches!(name.as_str(), "set" | "remove") && receiver_chain_contains_storage(&m.receiver)
}

#[derive(Default)]
struct AllowanceScan {
    has_storage_read: bool,
    has_storage_write: bool,
}

impl<'ast> Visit<'ast> for AllowanceScan {
    fn visit_expr_method_call(&mut self, i: &'ast ExprMethodCall) {
        if is_storage_get(i) {
            self.has_storage_read = true;
        } else if is_storage_write(i) {
            self.has_storage_write = true;
        }
        visit::visit_expr_method_call(self, i);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Check;
    use syn::parse_file;

    #[test]
    fn flags_transfer_from_without_allowance_clear() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Address, Env};

pub struct C;

#[contractimpl]
impl C {
    pub fn transfer_from(env: Env, from: Address, to: Address, amount: u128) {
        let allowance = env.storage().instance().get(&"allowance").unwrap_or(0);
        let new_balance = amount - allowance;
    }
}
"#,
        )?;
        let hits = AllowanceClearCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].severity, Severity::High);
        Ok(())
    }

    #[test]
    fn passes_when_allowance_cleared() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Address, Env};

pub struct C;

#[contractimpl]
impl C {
    pub fn transfer_from(env: Env, from: Address, to: Address, amount: u128) {
        let allowance = env.storage().instance().get(&"allowance").unwrap_or(0);
        env.storage().instance().set(&"allowance", &0);
    }
}
"#,
        )?;
        let hits = AllowanceClearCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }

    #[test]
    fn passes_when_allowance_removed() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Address, Env};

pub struct C;

#[contractimpl]
impl C {
    pub fn transfer_from(env: Env, from: Address, to: Address, amount: u128) {
        let allowance = env.storage().instance().get(&"allowance").unwrap_or(0);
        env.storage().instance().remove(&"allowance");
    }
}
"#,
        )?;
        let hits = AllowanceClearCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }
}
