//! Crypto hash of user input used directly as a storage key.
//!
//! `env.storage()...set(sha256(user_input), value)` allows an attacker to
//! pre-compute keys and overwrite arbitrary storage slots by crafting inputs.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ExprMethodCall, File, Pat};

const CHECK_NAME: &str = "hash-as-storage-key";

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

fn receiver_chain_contains_crypto(expr: &Expr) -> bool {
    match expr {
        Expr::MethodCall(m) => {
            if m.method == "crypto" {
                return true;
            }
            receiver_chain_contains_crypto(&m.receiver)
        }
        Expr::Field(f) => receiver_chain_contains_crypto(&f.base),
        _ => false,
    }
}

fn is_hash_call(m: &ExprMethodCall) -> bool {
    matches!(m.method.to_string().as_str(), "sha256" | "keccak256")
        && receiver_chain_contains_crypto(&m.receiver)
}

fn expr_is_hash_call(expr: &Expr) -> bool {
    match expr {
        Expr::MethodCall(m) => is_hash_call(m),
        Expr::Reference(r) => expr_is_hash_call(&r.expr),
        _ => false,
    }
}

/// Collect binding names assigned from hash calls.
fn collect_hash_bindings(block: &syn::Block) -> Vec<String> {
    let mut c = HashBindingCollector { bindings: vec![] };
    c.visit_block(block);
    c.bindings
}

struct HashBindingCollector {
    bindings: Vec<String>,
}

impl<'ast> Visit<'ast> for HashBindingCollector {
    fn visit_local(&mut self, i: &'ast syn::Local) {
        if let Some(init) = &i.init {
            let mut f = HashFinder { found: false };
            f.visit_expr(&init.expr);
            if f.found {
                if let Pat::Ident(pi) = &i.pat {
                    self.bindings.push(pi.ident.to_string());
                }
            }
        }
        visit::visit_local(self, i);
    }
}

struct HashFinder {
    found: bool,
}

impl<'ast> Visit<'ast> for HashFinder {
    fn visit_expr_method_call(&mut self, i: &'ast ExprMethodCall) {
        if is_hash_call(i) {
            self.found = true;
        }
        visit::visit_expr_method_call(self, i);
    }
}

fn key_arg_is_hash_or_binding(arg: &Expr, bindings: &[String]) -> bool {
    let inner = match arg {
        Expr::Reference(r) => &*r.expr,
        other => other,
    };
    // Inline hash call as key.
    if expr_is_hash_call(inner) {
        return true;
    }
    // Bound hash result used as key.
    if let Expr::Path(p) = inner {
        if let Some(seg) = p.path.segments.last() {
            return bindings.contains(&seg.ident.to_string());
        }
    }
    false
}

struct HashKeyVisitor<'a> {
    fn_name: String,
    bindings: Vec<String>,
    out: &'a mut Vec<Finding>,
}

impl<'ast> Visit<'ast> for HashKeyVisitor<'ast> {
    fn visit_expr_method_call(&mut self, i: &'ast ExprMethodCall) {
        if i.method == "set" && receiver_chain_contains_storage(&i.receiver) {
            if let Some(key_arg) = i.args.first() {
                if key_arg_is_hash_or_binding(key_arg, &self.bindings) {
                    self.out.push(Finding {
                        check_name: CHECK_NAME.to_string(),
                        severity: Severity::Medium,
                        file_path: String::new(),
                        line: i.span().start().line,
                        function_name: self.fn_name.clone(),
                        description: format!(
                            "Method `{}` uses a crypto hash (`sha256`/`keccak256`) of \
                             user-controlled input directly as a storage key. An attacker \
                             can pre-compute keys to overwrite arbitrary storage slots. \
                             Use a namespaced or typed key instead.",
                            self.fn_name
                        ),
                    });
                }
            }
        }
        visit::visit_expr_method_call(self, i);
    }
}

pub struct HashAsStorageKeyCheck;

impl Check for HashAsStorageKeyCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        for method in contractimpl_functions(file) {
            let fn_name = method.sig.ident.to_string();
            let bindings = collect_hash_bindings(&method.block);
            let mut v = HashKeyVisitor {
                fn_name,
                bindings,
                out: &mut out,
            };
            v.visit_block(&method.block);
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
    fn flags_inline_sha256_as_key() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, Bytes, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn store(env: Env, input: Bytes, val: u32) {
        env.storage().persistent().set(&env.crypto().sha256(&input), &val);
    }
}
"#;
        let file = parse_file(src)?;
        let hits = HashAsStorageKeyCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].severity, Severity::Medium);
        Ok(())
    }

    #[test]
    fn flags_bound_hash_as_key() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, Bytes, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn store(env: Env, input: Bytes, val: u32) {
        let key = env.crypto().keccak256(&input);
        env.storage().persistent().set(&key, &val);
    }
}
"#;
        let file = parse_file(src)?;
        let hits = HashAsStorageKeyCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        Ok(())
    }

    #[test]
    fn no_finding_for_constant_key() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, symbol_short, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn store(env: Env, val: u32) {
        env.storage().persistent().set(&symbol_short!("data"), &val);
    }
}
"#;
        let file = parse_file(src)?;
        let hits = HashAsStorageKeyCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }

    #[test]
    fn no_finding_for_hash_used_as_value() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, symbol_short, Bytes, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn store(env: Env, input: Bytes) {
        let hash = env.crypto().sha256(&input);
        env.storage().persistent().set(&symbol_short!("h"), &hash);
    }
}
"#;
        let file = parse_file(src)?;
        let hits = HashAsStorageKeyCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }
}
