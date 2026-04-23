//! Flags `env.invoke_contract(...)` calls that appear before `env.storage()...set(...)` writes.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ExprMethodCall, File};

const CHECK_NAME: &str = "reentrancy-before-storage-write";

pub struct ReentrancyCheck;

impl Check for ReentrancyCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        for method in contractimpl_functions(file) {
            let fn_name = method.sig.ident.to_string();
            let mut v = OrderCollector::default();
            v.visit_block(&method.block);

            let has_invoke_before_write = v
                .invoke_lines
                .iter()
                .any(|&inv| v.write_lines.iter().any(|&wr| inv < wr));

            if has_invoke_before_write {
                let line = *v.invoke_lines.iter().min().unwrap();
                out.push(Finding {
                    check_name: CHECK_NAME.to_string(),
                    severity: Severity::High,
                    file_path: String::new(),
                    line,
                    function_name: fn_name.clone(),
                    description: format!(
                        "Method `{fn_name}` calls `env.invoke_contract()` before a storage write. \
                         A malicious callee can re-enter this contract before state is committed."
                    ),
                });
            }
        }
        out
    }
}

fn is_invoke_contract(m: &ExprMethodCall) -> bool {
    m.method == "invoke_contract"
        && matches!(&*m.receiver, Expr::Path(p) if p.path.is_ident("env"))
}

fn receiver_has_storage(expr: &Expr) -> bool {
    match expr {
        Expr::MethodCall(m) => m.method == "storage" || receiver_has_storage(&m.receiver),
        _ => false,
    }
}

fn is_storage_write(m: &ExprMethodCall) -> bool {
    matches!(m.method.to_string().as_str(), "set" | "remove" | "bump" | "extend_ttl")
        && receiver_has_storage(&m.receiver)
}

#[derive(Default)]
struct OrderCollector {
    invoke_lines: Vec<usize>,
    write_lines: Vec<usize>,
}

impl<'ast> Visit<'ast> for OrderCollector {
    fn visit_expr_method_call(&mut self, i: &'ast ExprMethodCall) {
        if is_invoke_contract(i) {
            self.invoke_lines.push(i.span().start().line);
        }
        if is_storage_write(i) {
            self.write_lines.push(i.span().start().line);
        }
        visit::visit_expr_method_call(self, i);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_file;

    fn run(src: &str) -> Vec<Finding> {
        ReentrancyCheck.run(&parse_file(src).unwrap(), src)
    }

    #[test]
    fn flags_invoke_before_write() {
        let hits = run(r#"
pub struct C;
#[contractimpl]
impl C {
    pub fn transfer(env: Env, to: Address, amount: i128) {
        env.invoke_contract(&to, &symbol_short!("cb"), &());
        env.storage().persistent().set(&KEY, &amount);
    }
}
"#);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].severity, Severity::High);
        assert_eq!(hits[0].check_name, CHECK_NAME);
    }

    #[test]
    fn passes_write_before_invoke() {
        let hits = run(r#"
pub struct C;
#[contractimpl]
impl C {
    pub fn transfer(env: Env, to: Address, amount: i128) {
        env.storage().persistent().set(&KEY, &amount);
        env.invoke_contract(&to, &symbol_short!("cb"), &());
    }
}
"#);
        assert!(hits.is_empty());
    }

    #[test]
    fn ignores_non_contractimpl() {
        let hits = run(r#"
pub struct C;
impl C {
    pub fn transfer(env: Env, to: Address) {
        env.invoke_contract(&to, &symbol_short!("cb"), &());
        env.storage().persistent().set(&KEY, &1i128);
    }
}
"#);
        assert!(hits.is_empty());
    }
}
