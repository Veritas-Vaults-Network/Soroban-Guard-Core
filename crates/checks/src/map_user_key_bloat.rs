//! Map with user-supplied keys without size limit (map bloat).
//!
//! Inserting into a soroban_sdk::Map stored in instance or persistent storage using a key
//! derived from user input without a maximum-entry guard allows an attacker to bloat the map
//! indefinitely, increasing read/write costs for all users.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ExprMethodCall, File, FnArg, Pat};

const CHECK_NAME: &str = "map-user-key-bloat";

/// Detects Map::set() calls where the key argument traces back to a function parameter
/// and there is no map.len() guard in the same function.
pub struct MapUserKeyBloatCheck;

impl Check for MapUserKeyBloatCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        for method in contractimpl_functions(file) {
            let fn_name = method.sig.ident.to_string();
            
            // Collect parameter names
            let mut param_names = Vec::new();
            for input in &method.sig.inputs {
                if let FnArg::Typed(pat_type) = input {
                    if let Pat::Ident(pat_ident) = &*pat_type.pat {
                        param_names.push(pat_ident.ident.to_string());
                    }
                }
            }
            
            // Scan for Map::set() calls and len() checks
            let mut v = MapSetVisitor {
                param_names: &param_names,
                map_set_with_user_key: Vec::new(),
                has_len_check: false,
            };
            v.visit_block(&method.block);
            
            // Report findings for Map::set() calls with user keys and no len() guard
            if !v.map_set_with_user_key.is_empty() && !v.has_len_check {
                for line in v.map_set_with_user_key {
                    out.push(Finding {
                        check_name: CHECK_NAME.to_string(),
                        severity: Severity::Medium,
                        file_path: String::new(),
                        line,
                        function_name: fn_name.clone(),
                        description: format!(
                            "Function `{}` calls Map::set() with a user-supplied key without a \
                             map.len() guard. An attacker can bloat the map indefinitely, \
                             increasing storage costs for all users. Add a maximum size check.",
                            fn_name
                        ),
                    });
                }
            }
        }
        out
    }
}

struct MapSetVisitor<'a> {
    param_names: &'a [String],
    map_set_with_user_key: Vec<usize>,
    has_len_check: bool,
}

impl Visit<'_> for MapSetVisitor<'_> {
    fn visit_expr_method_call(&mut self, i: &ExprMethodCall) {
        // Check for Map::set() calls (but not storage().*.set())
        if i.method == "set" && !i.args.is_empty() && !is_storage_set(i) {
            // Check if first argument (key) references a parameter
            if let Some(first_arg) = i.args.first() {
                if expr_references_params(first_arg, self.param_names) {
                    self.map_set_with_user_key.push(i.span().start().line);
                }
            }
        }
        
        // Check for len() method calls (size guard)
        if i.method == "len" {
            self.has_len_check = true;
        }
        
        visit::visit_expr_method_call(self, i);
    }
}

fn is_storage_set(m: &ExprMethodCall) -> bool {
    receiver_chain_contains(&m.receiver, "storage")
}

fn receiver_chain_contains(expr: &Expr, name: &str) -> bool {
    match expr {
        Expr::MethodCall(m) => {
            m.method == name || receiver_chain_contains(&m.receiver, name)
        }
        Expr::Field(f) => receiver_chain_contains(&f.base, name),
        _ => false,
    }
}

/// Check if an expression directly references any function parameter (not nested in method calls)
fn expr_references_params(expr: &Expr, param_names: &[String]) -> bool {
    match expr {
        Expr::Path(p) => {
            if let Some(ident) = p.path.get_ident() {
                let ident_str = ident.to_string();
                // Exclude "env" as it's commonly used for Symbol::new(&env, "key")
                return param_names.contains(&ident_str) && ident_str != "env";
            }
            false
        }
        Expr::Reference(r) => expr_references_params(&r.expr, param_names),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_file;

    #[test]
    fn detects_map_set_with_user_key_no_guard() {
        let code = r#"
use soroban_sdk::{contract, contractimpl, Env, Map, Symbol};

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn add_item(env: Env, user_key: Symbol, value: u32) {
        let mut map: Map<Symbol, u32> = env.storage().instance().get(&Symbol::new(&env, "map")).unwrap_or(Map::new(&env));
        map.set(user_key, value);
        env.storage().instance().set(&Symbol::new(&env, "map"), &map);
    }
}
        "#;
        let file = parse_file(code).unwrap();
        let check = MapUserKeyBloatCheck;
        let findings = check.run(&file, code);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Medium);
        assert_eq!(findings[0].check_name, CHECK_NAME);
    }

    #[test]
    fn allows_map_set_with_len_guard() {
        let code = r#"
use soroban_sdk::{contract, contractimpl, Env, Map, Symbol};

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn add_item(env: Env, user_key: Symbol, value: u32) {
        let mut map: Map<Symbol, u32> = env.storage().instance().get(&Symbol::new(&env, "map")).unwrap_or(Map::new(&env));
        if map.len() >= 100 {
            panic!("Map is full");
        }
        map.set(user_key, value);
        env.storage().instance().set(&Symbol::new(&env, "map"), &map);
    }
}
        "#;
        let file = parse_file(code).unwrap();
        let check = MapUserKeyBloatCheck;
        let findings = check.run(&file, code);
        assert!(findings.is_empty());
    }

    #[test]
    fn allows_map_set_with_literal_key() {
        let code = r#"
use soroban_sdk::{contract, contractimpl, Env, Map, Symbol};

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn init(env: Env) {
        let mut map: Map<Symbol, u32> = Map::new(&env);
        map.set(Symbol::new(&env, "fixed_key"), 42);
        env.storage().instance().set(&Symbol::new(&env, "map"), &map);
    }
}
        "#;
        let file = parse_file(code).unwrap();
        let check = MapUserKeyBloatCheck;
        let findings = check.run(&file, code);
        assert!(findings.is_empty());
    }
}
