//! Map::get result unwrapped without a prior has() guard.
//!
//! Calling `map.get(&key).unwrap()` (or `.expect(...)`) without first checking
//! `map.has(&key)` will panic at runtime if the key is absent. This is especially
//! dangerous when the key is user-supplied.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ExprMethodCall, File};

const CHECK_NAME: &str = "map-get-unwrap";

pub struct MapGetUnwrapCheck;

impl Check for MapGetUnwrapCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        for method in contractimpl_functions(file) {
            let mut scan = MapGetScan::default();
            scan.visit_block(&method.block);
            for line in scan.unguarded_unwraps {
                out.push(Finding {
                    check_name: CHECK_NAME.to_string(),
                    severity: Severity::Medium,
                    file_path: String::new(),
                    line,
                    function_name: method.sig.ident.to_string(),
                    description: "Calling `.get(&key).unwrap()` on a Map without a prior \
                        `.has(&key)` check will panic if the key is absent."
                        .to_string(),
                });
            }
        }
        out
    }
}

/// Returns true if the expression looks like a Map variable (not a storage chain).
/// We accept any receiver that is NOT a `.storage()` chain — i.e. a local variable
/// or field that could be a `soroban_sdk::Map`.
fn is_map_receiver(expr: &Expr) -> bool {
    match expr {
        // Reject storage chains: env.storage().instance() / .temporary() / .persistent()
        Expr::MethodCall(m) => {
            let method = m.method.to_string();
            if matches!(method.as_str(), "instance" | "temporary" | "persistent" | "storage") {
                return false;
            }
            // A method call result used as receiver (e.g. `self.get_map()`) is fine
            true
        }
        Expr::Path(_) | Expr::Field(_) => true,
        _ => false,
    }
}

fn is_map_get_unwrap(m: &ExprMethodCall) -> bool {
    if m.method != "unwrap" && m.method != "expect" {
        return false;
    }
    match &*m.receiver {
        Expr::MethodCall(inner) => inner.method == "get" && is_map_receiver(&inner.receiver),
        _ => false,
    }
}

fn is_map_has(m: &ExprMethodCall) -> bool {
    m.method == "has" && is_map_receiver(&m.receiver)
}

#[derive(Default)]
struct MapGetScan {
    has_guard: bool,
    unguarded_unwraps: Vec<usize>,
}

impl<'ast> Visit<'ast> for MapGetScan {
    fn visit_expr_method_call(&mut self, i: &'ast ExprMethodCall) {
        if is_map_has(i) {
            self.has_guard = true;
        } else if is_map_get_unwrap(i) && !self.has_guard {
            self.unguarded_unwraps.push(i.span().start().line);
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
    fn flags_map_get_unwrap_without_has() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env, Map};

pub struct C;

#[contractimpl]
impl C {
    pub fn lookup(env: Env, map: Map<u32, u32>, key: u32) -> u32 {
        map.get(&key).unwrap()
    }
}
"#,
        )?;
        let hits = MapGetUnwrapCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].severity, Severity::Medium);
        Ok(())
    }

    #[test]
    fn flags_map_get_expect_without_has() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env, Map};

pub struct C;

#[contractimpl]
impl C {
    pub fn lookup(env: Env, map: Map<u32, u32>, key: u32) -> u32 {
        map.get(&key).expect("missing")
    }
}
"#,
        )?;
        let hits = MapGetUnwrapCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        Ok(())
    }

    #[test]
    fn passes_when_has_guard_present() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env, Map};

pub struct C;

#[contractimpl]
impl C {
    pub fn lookup(env: Env, map: Map<u32, u32>, key: u32) -> u32 {
        if map.has(&key) {
            map.get(&key).unwrap()
        } else {
            0
        }
    }
}
"#,
        )?;
        let hits = MapGetUnwrapCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }

    #[test]
    fn does_not_flag_storage_get_unwrap() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env};

pub struct C;

#[contractimpl]
impl C {
    pub fn get_val(env: Env) -> u32 {
        env.storage().instance().get(&"key").unwrap()
    }
}
"#,
        )?;
        // instance-get-without-guard handles this; map-get-unwrap must not double-flag it
        let hits = MapGetUnwrapCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }
}
