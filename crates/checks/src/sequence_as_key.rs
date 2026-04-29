//! Detects storage keys derived from ledger sequence or timestamp.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ExprMethodCall, File};

const CHECK_NAME: &str = "sequence-as-key";

/// Flags storage set calls where the key is derived from env.ledger().sequence() or timestamp().
pub struct SequenceAsKeyCheck;

impl Check for SequenceAsKeyCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        for method in contractimpl_functions(file) {
            let fn_name = method.sig.ident.to_string();
            let mut v = SequenceVisitor {
                fn_name: fn_name.clone(),
                out: &mut out,
            };
            v.visit_block(&method.block);
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

fn receiver_chain_contains_persistent_or_instance(expr: &Expr) -> bool {
    match expr {
        Expr::MethodCall(m) => {
            if m.method == "persistent" || m.method == "instance" {
                return true;
            }
            receiver_chain_contains_persistent_or_instance(&m.receiver)
        }
        Expr::Field(f) => receiver_chain_contains_persistent_or_instance(&f.base),
        _ => false,
    }
}

fn is_persistent_or_instance_set_call(m: &ExprMethodCall) -> bool {
    m.method == "set"
        && receiver_chain_contains_storage(&m.receiver)
        && receiver_chain_contains_persistent_or_instance(&m.receiver)
}

fn expr_contains_ledger_sequence_or_timestamp(expr: &Expr) -> bool {
    match expr {
        Expr::MethodCall(m) => {
            if m.method == "sequence" || m.method == "timestamp" {
                return receiver_chain_contains_ledger(&m.receiver);
            }
            m.args
                .iter()
                .any(expr_contains_ledger_sequence_or_timestamp)
                || expr_contains_ledger_sequence_or_timestamp(&m.receiver)
        }
        Expr::Call(c) => c
            .args
            .iter()
            .any(expr_contains_ledger_sequence_or_timestamp),
        Expr::Field(f) => expr_contains_ledger_sequence_or_timestamp(&f.base),
        Expr::Index(i) => {
            expr_contains_ledger_sequence_or_timestamp(&i.expr)
                || expr_contains_ledger_sequence_or_timestamp(&i.index)
        }
        Expr::Binary(b) => {
            expr_contains_ledger_sequence_or_timestamp(&b.left)
                || expr_contains_ledger_sequence_or_timestamp(&b.right)
        }
        Expr::Unary(u) => expr_contains_ledger_sequence_or_timestamp(&u.expr),
        Expr::Paren(p) => expr_contains_ledger_sequence_or_timestamp(&p.expr),
        Expr::Reference(r) => expr_contains_ledger_sequence_or_timestamp(&r.expr),
        Expr::Tuple(t) => t
            .elems
            .iter()
            .any(expr_contains_ledger_sequence_or_timestamp),
        Expr::Array(a) => a
            .elems
            .iter()
            .any(expr_contains_ledger_sequence_or_timestamp),
        Expr::Struct(s) => s
            .fields
            .iter()
            .any(|f| expr_contains_ledger_sequence_or_timestamp(&f.expr)),
        _ => false,
    }
}

fn receiver_chain_contains_ledger(expr: &Expr) -> bool {
    match expr {
        Expr::MethodCall(m) => {
            if m.method == "ledger" {
                return true;
            }
            receiver_chain_contains_ledger(&m.receiver)
        }
        Expr::Field(f) => receiver_chain_contains_ledger(&f.base),
        _ => false,
    }
}

struct SequenceVisitor<'a> {
    fn_name: String,
    out: &'a mut Vec<Finding>,
}

impl Visit<'_> for SequenceVisitor<'_> {
    fn visit_expr_method_call(&mut self, i: &ExprMethodCall) {
        if is_persistent_or_instance_set_call(i) {
            if let Some(key_arg) = i.args.first() {
                if expr_contains_ledger_sequence_or_timestamp(key_arg) {
                    self.out.push(Finding {
                        check_name: CHECK_NAME.to_string(),
                        severity: Severity::Medium,
                        file_path: String::new(),
                        line: i.span().start().line,
                        function_name: self.fn_name.clone(),
                        description: format!(
                            "Storage key in `{}` is derived from `env.ledger().sequence()` or `timestamp()`. \
                             This creates a new entry every ledger, causing unbounded growth and making old entries irretrievable.",
                            self.fn_name
                        ),
                    });
                }
            }
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
    fn flags_persistent_set_with_sequence_key() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env};

pub struct C;

#[contractimpl]
impl C {
    pub fn store(env: Env) {
        env.storage().persistent().set(&env.ledger().sequence(), &42u32);
    }
}
"#,
        )?;
        let hits = SequenceAsKeyCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].severity, Severity::Medium);
        assert!(hits[0].description.contains("sequence"));
        Ok(())
    }

    #[test]
    fn flags_instance_set_with_timestamp_key() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env};

pub struct C;

#[contractimpl]
impl C {
    pub fn store(env: Env) {
        env.storage().instance().set(&env.ledger().timestamp(), &42u32);
    }
}
"#,
        )?;
        let hits = SequenceAsKeyCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        assert!(hits[0].description.contains("timestamp"));
        Ok(())
    }

    #[test]
    fn ignores_temporary_set_with_sequence() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env};

pub struct C;

#[contractimpl]
impl C {
    pub fn store(env: Env) {
        let seq = env.ledger().sequence();
        env.storage().temporary().set(&seq, &42u32);
    }
}
"#,
        )?;
        let hits = SequenceAsKeyCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }

    #[test]
    fn ignores_persistent_set_with_static_key() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, symbol_short, Env};

pub struct C;

const K: soroban_sdk::Symbol = symbol_short!("k");

#[contractimpl]
impl C {
    pub fn store(env: Env) {
        env.storage().persistent().set(&K, &42u32);
    }
}
"#,
        )?;
        let hits = SequenceAsKeyCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }
}
