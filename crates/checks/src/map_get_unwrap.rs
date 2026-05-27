//! Detects Map::get().unwrap() without a preceding Map::has() guard.

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
            let mut scan = MapGetScan {
                fn_name: method.sig.ident.to_string(),
                has_guard: false,
                out: &mut out,
            };
            scan.visit_block(&method.block);
        }
        out
    }
}

/// Returns true if the method call is `<expr>.get(...).unwrap()` or `.expect(...)`.
fn is_map_get_unwrap(m: &ExprMethodCall) -> bool {
    if m.method != "unwrap" && m.method != "expect" {
        return false;
    }
    matches!(&*m.receiver, Expr::MethodCall(inner) if inner.method == "get")
}

/// Returns true if the method call is `<expr>.has(...)`.
fn is_map_has(m: &ExprMethodCall) -> bool {
    m.method == "has"
}

struct MapGetScan<'a> {
    fn_name: String,
    has_guard: bool,
    out: &'a mut Vec<Finding>,
}

impl Visit<'_> for MapGetScan<'_> {
    fn visit_expr_method_call(&mut self, i: &ExprMethodCall) {
        if is_map_has(i) {
            self.has_guard = true;
        } else if is_map_get_unwrap(i) && !self.has_guard {
            self.out.push(Finding {
                check_name: CHECK_NAME.to_string(),
                severity: Severity::Medium,
                file_path: String::new(),
                line: i.span().start().line,
                function_name: self.fn_name.clone(),
                description: format!(
                    "`map.get().unwrap()` in `{}` is not guarded by a preceding `map.has()` check. \
                     Calling `.unwrap()` on an absent key panics and can be exploited with user-supplied keys.",
                    self.fn_name
                ),
            });
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
    fn flags_get_unwrap_without_has() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env, Map};

pub struct C;

#[contractimpl]
impl C {
    pub fn get_value(_env: Env, map: Map<u32, u32>, key: u32) -> u32 {
        map.get(&key).unwrap()
    }
}
"#,
        )?;
        let hits = MapGetUnwrapCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].severity, Severity::Medium);
        assert_eq!(hits[0].function_name, "get_value");
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
    pub fn get_value(_env: Env, map: Map<u32, u32>, key: u32) -> u32 {
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
    fn flags_expect_without_has() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env, Map};

pub struct C;

#[contractimpl]
impl C {
    pub fn get_value(_env: Env, map: Map<u32, u32>, key: u32) -> u32 {
        map.get(&key).expect("missing key")
    }
}
"#,
        )?;
        let hits = MapGetUnwrapCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        Ok(())
    }
}
