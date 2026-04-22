//! Detects unbounded push/insert operations on storage collections.
//!
//! Appending to a Vec or Map stored in env.storage().instance() without a size cap
//! can cause the instance storage entry to grow unboundedly, eventually exceeding
//! ledger limits and bricking the contract.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ExprMethodCall, File};

const CHECK_NAME: &str = "unbounded-storage";

/// Flags `push_back`, `push_front`, or `insert` method calls on values retrieved
/// from `env.storage().instance()` without a preceding length/size check.
pub struct UnboundedStorageCheck;

impl Check for UnboundedStorageCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        for method in contractimpl_functions(file) {
            let fn_name = method.sig.ident.to_string();
            let mut scan = UnboundedStorageScan {
                fn_name,
                out: &mut out,
            };
            scan.visit_block(&method.block);
        }
        out
    }
}

struct UnboundedStorageScan<'a> {
    fn_name: String,
    out: &'a mut Vec<Finding>,
}

impl<'ast> Visit<'ast> for UnboundedStorageScan<'_> {
    fn visit_expr_method_call(&mut self, i: &'ast ExprMethodCall) {
        // Check for push_back, push_front, insert methods
        let method_name = i.method.to_string();
        if matches!(
            method_name.as_str(),
            "push_back" | "push_front" | "insert"
        ) {
            // Check if receiver is from instance storage
            if is_from_instance_storage(&i.receiver) {
                // For simplicity, we flag all unbounded push/insert operations.
                // A more sophisticated check would track whether there was a
                // preceding .len() or .capacity() check.
                let line = i.span().start().line;
                self.out.push(Finding {
                    check_name: CHECK_NAME.to_string(),
                    severity: Severity::Medium,
                    file_path: String::new(),
                    line,
                    function_name: self.fn_name.clone(),
                    description: format!(
                        "Method `{}` calls `{}` on a value from instance storage without \
                         an apparent size/capacity check. This can cause unbounded growth, \
                         exceeding ledger limits and bricking the contract.",
                        self.fn_name, method_name
                    ),
                });
            }
        }

        visit::visit_expr_method_call(self, i);
    }
}

fn is_from_instance_storage(expr: &Expr) -> bool {
    match expr {
        Expr::MethodCall(m) => {
            // Check if this is a .get() call on instance storage
            if m.method == "get" {
                if is_instance_storage_receiver(&m.receiver) {
                    return true;
                }
            }
            // Otherwise recurse
            is_from_instance_storage(&m.receiver)
        }
        Expr::Field(f) => is_from_instance_storage(&f.base),
        _ => false,
    }
}

fn is_instance_storage_receiver(expr: &Expr) -> bool {
    match expr {
        Expr::MethodCall(m) => {
            // Check for .instance() call
            if m.method == "instance" {
                return true;
            }
            is_instance_storage_receiver(&m.receiver)
        }
        Expr::Field(f) => is_instance_storage_receiver(&f.base),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_file;

    #[test]
    fn detects_unbounded_push_back() {
        let code = r#"
#[contractimpl]
impl MyContract {
    pub fn add_item(env: Env) {
        let mut items = env.storage().instance().get::<_, Vec<i32>>(&key).unwrap();
        items.push_back(42);
        env.storage().instance().set(&key, &items);
    }
}
        "#;
        let file = parse_file(code).unwrap();
        let check = UnboundedStorageCheck;
        let findings = check.run(&file, code);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].check_name, CHECK_NAME);
        assert_eq!(findings[0].severity, Severity::Medium);
    }

    #[test]
    fn detects_unbounded_insert() {
        let code = r#"
#[contractimpl]
impl MyContract {
    pub fn add_entry(env: Env) {
        let mut map = env.storage().instance().get::<_, Map<u32, String>>(&key).unwrap();
        map.insert(1, "value".into());
    }
}
        "#;
        let file = parse_file(code).unwrap();
        let check = UnboundedStorageCheck;
        let findings = check.run(&file, code);
        assert!(findings.len() >= 1);
    }

    #[test]
    fn allows_push_on_local_vec() {
        let code = r#"
#[contractimpl]
impl MyContract {
    pub fn process(env: Env) {
        let mut items = Vec::new();
        items.push_back(42);
    }
}
        "#;
        let file = parse_file(code).unwrap();
        let check = UnboundedStorageCheck;
        let findings = check.run(&file, code);
        // Should not flag local Vec operations (not from instance storage)
        assert!(findings.is_empty());
    }
}
