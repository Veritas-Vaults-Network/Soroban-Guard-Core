//! `Symbol::new` with a runtime (non-literal) string used as a storage key.
//!
//! `env.storage()...set(Symbol::new(&env, expr), ...)` where `expr` is not a
//! string literal allows callers to write to arbitrary storage slots.
//! All storage keys should be compile-time constants.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ExprCall, ExprMethodCall, File, Lit};

const CHECK_NAME: &str = "dynamic-symbol-key";

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

/// True if the expression is a `Symbol::new(...)` call.
fn is_symbol_new_call(expr: &Expr) -> Option<&ExprCall> {
    let Expr::Call(call) = expr else {
        return None;
    };
    let Expr::Path(p) = &*call.func else {
        return None;
    };
    let segs = &p.path.segments;
    if segs.len() == 2 && segs[0].ident == "Symbol" && segs[1].ident == "new" {
        return Some(call);
    }
    None
}

/// True if the second argument to `Symbol::new` is a string literal.
fn second_arg_is_string_lit(call: &ExprCall) -> bool {
    let Some(arg) = call.args.iter().nth(1) else {
        return false;
    };
    // Strip reference if present.
    let inner = match arg {
        Expr::Reference(r) => &*r.expr,
        other => other,
    };
    matches!(inner, Expr::Lit(syn::ExprLit { lit: Lit::Str(_), .. }))
}

/// True if the first argument to `.set(key, val)` (after stripping `&`) contains
/// a `Symbol::new` call with a non-literal second argument.
fn set_key_is_dynamic_symbol(m: &ExprMethodCall) -> bool {
    if m.method != "set" {
        return false;
    }
    if !receiver_chain_contains_storage(&m.receiver) {
        return false;
    }
    let Some(key_arg) = m.args.first() else {
        return false;
    };
    let inner = match key_arg {
        Expr::Reference(r) => &*r.expr,
        other => other,
    };
    if let Some(call) = is_symbol_new_call(inner) {
        return !second_arg_is_string_lit(call);
    }
    false
}

struct DynamicSymbolVisitor<'a> {
    fn_name: String,
    out: &'a mut Vec<Finding>,
}

impl Visit<'_> for DynamicSymbolVisitor<'_> {
    fn visit_expr_method_call(&mut self, i: &ExprMethodCall) {
        if set_key_is_dynamic_symbol(i) {
            self.out.push(Finding {
                check_name: CHECK_NAME.to_string(),
                severity: Severity::High,
                file_path: String::new(),
                line: i.span().start().line,
                function_name: self.fn_name.clone(),
                description: format!(
                    "Method `{}` calls `env.storage()...set()` with a `Symbol::new` key \
                     constructed from a runtime expression (not a string literal). Callers \
                     can supply arbitrary key strings to write to any storage slot. Use \
                     `symbol_short!` or a compile-time constant key instead.",
                    self.fn_name
                ),
            });
        }
        visit::visit_expr_method_call(self, i);
    }
}

pub struct DynamicSymbolKeyCheck;

impl Check for DynamicSymbolKeyCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        for method in contractimpl_functions(file) {
            let fn_name = method.sig.ident.to_string();
            let mut v = DynamicSymbolVisitor {
                fn_name,
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
    fn flags_symbol_new_with_param_key() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, Env, Symbol};
pub struct C;
#[contractimpl]
impl C {
    pub fn store(env: Env, key: soroban_sdk::String, val: u32) {
        env.storage().persistent().set(&Symbol::new(&env, key), &val);
    }
}
"#;
        let file = parse_file(src)?;
        let hits = DynamicSymbolKeyCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].severity, Severity::High);
        assert!(hits[0].description.contains("Symbol::new"));
        Ok(())
    }

    #[test]
    fn flags_symbol_new_with_variable_key() -> Result<(), syn::Error> {
        // Note: the set call here uses a variable `sym`, not Symbol::new inline.
        // This test verifies the inline case is caught.
        let src2 = r#"
use soroban_sdk::{contractimpl, Env, Symbol};
pub struct C;
#[contractimpl]
impl C {
    pub fn store(env: Env, tag: soroban_sdk::String) {
        env.storage().instance().set(&Symbol::new(&env, tag), &0u32);
    }
}
"#;
        let file = parse_file(src2)?;
        let hits = DynamicSymbolKeyCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        Ok(())
    }

    #[test]
    fn no_finding_for_string_literal_key() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, Env, Symbol};
pub struct C;
#[contractimpl]
impl C {
    pub fn store(env: Env, val: u32) {
        env.storage().persistent().set(&Symbol::new(&env, "fixed_key"), &val);
    }
}
"#;
        let file = parse_file(src)?;
        let hits = DynamicSymbolKeyCheck.run(&file, "");
        assert!(hits.is_empty(), "{hits:?}");
        Ok(())
    }

    #[test]
    fn no_finding_for_symbol_short_key() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, symbol_short, Env};
pub struct C;
const KEY: soroban_sdk::Symbol = symbol_short!("key");
#[contractimpl]
impl C {
    pub fn store(env: Env, val: u32) {
        env.storage().persistent().set(&KEY, &val);
    }
}
"#;
        let file = parse_file(src)?;
        let hits = DynamicSymbolKeyCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }

    #[test]
    fn no_finding_for_get_with_dynamic_symbol() -> Result<(), syn::Error> {
        // get() with dynamic symbol is not flagged by this check (only set)
        let src = r#"
use soroban_sdk::{contractimpl, Env, Symbol};
pub struct C;
#[contractimpl]
impl C {
    pub fn load(env: Env, key: soroban_sdk::String) -> Option<u32> {
        env.storage().persistent().get(&Symbol::new(&env, key))
    }
}
"#;
        let file = parse_file(src)?;
        let hits = DynamicSymbolKeyCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }
}
