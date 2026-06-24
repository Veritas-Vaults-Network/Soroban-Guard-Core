//! Detects init-like methods that write storage without first checking initialization state.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Block, Expr, ExprMethodCall, File};

const CHECK_NAME: &str = "double-init";

pub struct DoubleInitCheck;

impl Check for DoubleInitCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        for method in contractimpl_functions(file) {
            let fn_name = method.sig.ident.to_string();
            if !is_init_like(&fn_name) {
                continue;
            }

            let mut scan = FuncBodyScan::default();
            scan.visit_block(&method.block);
            if !scan.storage_set || scan.storage_guard {
                continue;
            }

            let line = first_storage_set_line(&method.block)
                .unwrap_or_else(|| method.sig.ident.span().start().line);
            out.push(Finding {
                check_name: CHECK_NAME.to_string(),
                severity: Severity::High,
                file_path: String::new(),
                line,
                function_name: fn_name.clone(),
                description: format!(
                    "Init-like method `{fn_name}` writes to storage without first checking \
                     whether initialization state already exists. Add a storage `has` or `get` \
                     guard before setting initialization data to prevent re-initialization."
                ),
            });
        }
        out
    }
}

fn is_init_like(name: &str) -> bool {
    name.to_ascii_lowercase().contains("init")
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

fn is_storage_guard_call(m: &ExprMethodCall) -> bool {
    matches!(m.method.to_string().as_str(), "has" | "get")
        && receiver_chain_contains_storage(&m.receiver)
}

#[derive(Default)]
struct FuncBodyScan {
    storage_set: bool,
    storage_guard: bool,
}

impl<'ast> Visit<'ast> for FuncBodyScan {
    fn visit_expr_method_call(&mut self, i: &'ast ExprMethodCall) {
        if is_storage_set_call(i) {
            self.storage_set = true;
        }
        if is_storage_guard_call(i) {
            self.storage_guard = true;
        }
        visit::visit_expr_method_call(self, i);
    }
}

struct FirstStorageSet {
    line: Option<usize>,
}

impl<'ast> Visit<'ast> for FirstStorageSet {
    fn visit_expr_method_call(&mut self, i: &'ast ExprMethodCall) {
        if self.line.is_none() && is_storage_set_call(i) {
            self.line = Some(i.span().start().line);
        }
        visit::visit_expr_method_call(self, i);
    }
}

fn first_storage_set_line(block: &Block) -> Option<usize> {
    let mut visitor = FirstStorageSet { line: None };
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
        Ok(DoubleInitCheck.run(&file, src))
    }

    #[test]
    fn flags_init_storage_set_without_guard() -> Result<(), syn::Error> {
        let hits = run(r#"
use soroban_sdk::{contractimpl, Address, Env, Symbol};

pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn initialize(env: Env, owner: Address) {
        env.storage().instance().set(&Symbol::new(&env, "owner"), &owner);
    }
}
"#)?;

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].check_name, CHECK_NAME);
        assert_eq!(hits[0].severity, Severity::High);
        assert_eq!(hits[0].function_name, "initialize");
        Ok(())
    }

    #[test]
    fn passes_when_init_checks_storage_first() -> Result<(), syn::Error> {
        let hits = run(r#"
use soroban_sdk::{contractimpl, Address, Env, Symbol};

pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn init(env: Env, owner: Address) {
        let key = Symbol::new(&env, "owner");
        if env.storage().instance().has(&key) {
            panic!("already initialized");
        }
        env.storage().instance().set(&key, &owner);
    }
}
"#)?;

        assert!(hits.is_empty());
        Ok(())
    }

    #[test]
    fn passes_when_init_reads_storage_first() -> Result<(), syn::Error> {
        let hits = run(r#"
use soroban_sdk::{contractimpl, Address, Env, Symbol};

pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn initialize(env: Env, owner: Address) {
        let key = Symbol::new(&env, "owner");
        let existing: Option<Address> = env.storage().instance().get(&key);
        if existing.is_some() {
            panic!("already initialized");
        }
        env.storage().instance().set(&key, &owner);
    }
}
"#)?;

        assert!(hits.is_empty());
        Ok(())
    }

    #[test]
    fn ignores_non_init_storage_set() -> Result<(), syn::Error> {
        let hits = run(r#"
use soroban_sdk::{contractimpl, Address, Env, Symbol};

pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn set_owner(env: Env, owner: Address) {
        env.storage().instance().set(&Symbol::new(&env, "owner"), &owner);
    }
}
"#)?;

        assert!(hits.is_empty());
        Ok(())
    }
}
