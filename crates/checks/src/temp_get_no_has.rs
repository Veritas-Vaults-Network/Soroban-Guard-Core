//! Detects temporary storage get calls without preceding has checks.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ExprMethodCall, File};

const CHECK_NAME: &str = "temp-get-no-has";

/// Flags env.storage().temporary().get() calls not guarded by has() in the same function.
pub struct TempGetNoHasCheck;

impl Check for TempGetNoHasCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        for method in contractimpl_functions(file) {
            let fn_name = method.sig.ident.to_string();
            let mut v = TempVisitor {
                fn_name: fn_name.clone(),
                has_temporary_has: false,
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

fn receiver_chain_contains_temporary(expr: &Expr) -> bool {
    match expr {
        Expr::MethodCall(m) => {
            if m.method == "temporary" {
                return true;
            }
            receiver_chain_contains_temporary(&m.receiver)
        }
        Expr::Field(f) => receiver_chain_contains_temporary(&f.base),
        _ => false,
    }
}

fn is_temporary_get_call(m: &ExprMethodCall) -> bool {
    m.method == "get"
        && receiver_chain_contains_storage(&m.receiver)
        && receiver_chain_contains_temporary(&m.receiver)
}

fn is_temporary_has_call(m: &ExprMethodCall) -> bool {
    m.method == "has"
        && receiver_chain_contains_storage(&m.receiver)
        && receiver_chain_contains_temporary(&m.receiver)
}

struct TempVisitor<'a> {
    fn_name: String,
    has_temporary_has: bool,
    out: &'a mut Vec<Finding>,
}

impl Visit<'_> for TempVisitor<'_> {
    fn visit_expr_method_call(&mut self, i: &ExprMethodCall) {
        if is_temporary_has_call(i) {
            self.has_temporary_has = true;
        } else if is_temporary_get_call(i) && !self.has_temporary_has {
            self.out.push(Finding {
                check_name: CHECK_NAME.to_string(),
                severity: Severity::Medium,
                file_path: String::new(),
                line: i.span().start().line,
                function_name: self.fn_name.clone(),
                description: format!(
                    "`env.storage().temporary().get()` in `{}` is not guarded by a preceding `has()` check. \
                     Temporary storage entries can expire silently, so always check `has()` before `get()`.",
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
    fn flags_temporary_get_without_has() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env};

pub struct C;

#[contractimpl]
impl C {
    pub fn get_without_has(env: Env) {
        let key = 1u32;
        let _val = env.storage().temporary().get(&key);
    }
}
"#,
        )?;
        let hits = TempGetNoHasCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].severity, Severity::Medium);
        assert!(hits[0].description.contains("not guarded"));
        Ok(())
    }

    #[test]
    fn allows_temporary_get_with_has() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env};

pub struct C;

#[contractimpl]
impl C {
    pub fn get_with_has(env: Env) {
        let key = 1u32;
        if env.storage().temporary().has(&key) {
            let _val = env.storage().temporary().get(&key);
        }
    }
}
"#,
        )?;
        let hits = TempGetNoHasCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }

    #[test]
    fn ignores_persistent_get() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env};

pub struct C;

#[contractimpl]
impl C {
    pub fn persistent_get(env: Env) {
        let key = 1u32;
        let _val = env.storage().persistent().get(&key);
    }
}
"#,
        )?;
        let hits = TempGetNoHasCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }
}
