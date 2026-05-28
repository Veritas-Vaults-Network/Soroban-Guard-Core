//! `invoke_contract` return value cast without error handling.
//!
//! `env.invoke_contract(...)` returns a generic `Val`. Casting it directly via
//! `.unwrap()` or `.expect()` on `try_into_val` / `from_val` silently produces
//! a zero/default value on type mismatch, leading to logic errors.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ExprMethodCall, File, Pat};

const CHECK_NAME: &str = "invoke-unchecked-cast";

fn is_invoke_contract(m: &ExprMethodCall) -> bool {
    m.method == "invoke_contract"
}

fn is_unchecked_unwrap(m: &ExprMethodCall) -> bool {
    matches!(m.method.to_string().as_str(), "unwrap" | "expect")
}

fn is_cast_method(m: &ExprMethodCall) -> bool {
    matches!(
        m.method.to_string().as_str(),
        "try_into_val" | "from_val" | "try_from_val"
    )
}

/// True if the expression chain is: <invoke_contract_result>.try_into_val(...).unwrap()
/// or <invoke_contract_result>.from_val(...).unwrap()
fn is_unchecked_cast_chain(expr: &Expr) -> bool {
    let Expr::MethodCall(outer) = expr else {
        return false;
    };
    if !is_unchecked_unwrap(outer) {
        return false;
    }
    // The receiver of unwrap() must be a cast method call.
    let Expr::MethodCall(cast) = &*outer.receiver else {
        return false;
    };
    if !is_cast_method(cast) {
        return false;
    }
    // The receiver of the cast must be (or contain) invoke_contract.
    expr_contains_invoke_contract(&cast.receiver)
}

fn expr_contains_invoke_contract(expr: &Expr) -> bool {
    match expr {
        Expr::MethodCall(m) => {
            if is_invoke_contract(m) {
                return true;
            }
            expr_contains_invoke_contract(&m.receiver)
        }
        Expr::Reference(r) => expr_contains_invoke_contract(&r.expr),
        _ => false,
    }
}

/// Collect binding names assigned from invoke_contract.
fn collect_invoke_bindings(block: &syn::Block) -> Vec<String> {
    let mut c = InvokeBindingCollector { bindings: vec![] };
    c.visit_block(block);
    c.bindings
}

struct InvokeBindingCollector {
    bindings: Vec<String>,
}

impl<'ast> Visit<'ast> for InvokeBindingCollector {
    fn visit_local(&mut self, i: &'ast syn::Local) {
        if let Some(init) = &i.init {
            let mut f = InvokeFinder { found: false };
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

struct InvokeFinder {
    found: bool,
}

impl<'ast> Visit<'ast> for InvokeFinder {
    fn visit_expr_method_call(&mut self, i: &'ast ExprMethodCall) {
        if is_invoke_contract(i) {
            self.found = true;
        }
        visit::visit_expr_method_call(self, i);
    }
}

/// Check if any binding from invoke_contract is later cast with unwrap/expect.
fn binding_is_unchecked_cast(block: &syn::Block, bindings: &[String]) -> Option<usize> {
    let mut checker = UncheckedCastChecker {
        bindings,
        line: None,
    };
    checker.visit_block(block);
    checker.line
}

struct UncheckedCastChecker<'a> {
    bindings: &'a [String],
    line: Option<usize>,
}

impl<'ast> Visit<'ast> for UncheckedCastChecker<'_> {
    fn visit_expr_method_call(&mut self, i: &'ast ExprMethodCall) {
        if self.line.is_some() {
            return;
        }
        // Pattern: binding.try_into_val(...).unwrap()
        if is_unchecked_unwrap(i) {
            if let Expr::MethodCall(cast) = &*i.receiver {
                if is_cast_method(cast) {
                    if let Expr::Path(p) = &*cast.receiver {
                        if let Some(seg) = p.path.segments.last() {
                            if self.bindings.contains(&seg.ident.to_string()) {
                                self.line = Some(i.span().start().line);
                                return;
                            }
                        }
                    }
                }
            }
        }
        visit::visit_expr_method_call(self, i);
    }
}

pub struct InvokeUncheckedCastCheck;

impl Check for InvokeUncheckedCastCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        for method in contractimpl_functions(file) {
            let fn_name = method.sig.ident.to_string();

            // Check for inline unchecked cast chains.
            let mut inline = InlineUncheckedCastVisitor {
                fn_name: fn_name.clone(),
                out: &mut out,
            };
            inline.visit_block(&method.block);

            // Check for bound-then-cast patterns.
            let bindings = collect_invoke_bindings(&method.block);
            if !bindings.is_empty() {
                if let Some(line) = binding_is_unchecked_cast(&method.block, &bindings) {
                    out.push(Finding {
                        check_name: CHECK_NAME.to_string(),
                        severity: Severity::Medium,
                        file_path: String::new(),
                        line,
                        function_name: fn_name.clone(),
                        description: format!(
                            "Method `{fn_name}` casts the result of `invoke_contract` via \
                             `try_into_val`/`from_val` with `.unwrap()` or `.expect()`. A \
                             type mismatch silently produces a zero/default value. Use a \
                             `match` or `if let Ok(...)` to handle the error explicitly."
                        ),
                    });
                }
            }
        }
        out
    }
}

struct InlineUncheckedCastVisitor<'a> {
    fn_name: String,
    out: &'a mut Vec<Finding>,
}

impl<'ast> Visit<'ast> for InlineUncheckedCastVisitor<'ast> {
    fn visit_expr_method_call(&mut self, i: &'ast ExprMethodCall) {
        if is_unchecked_cast_chain(&Expr::MethodCall(i.clone())) {
            self.out.push(Finding {
                check_name: CHECK_NAME.to_string(),
                severity: Severity::Medium,
                file_path: String::new(),
                line: i.span().start().line,
                function_name: self.fn_name.clone(),
                description: format!(
                    "Method `{}` casts the result of `invoke_contract` via \
                     `try_into_val`/`from_val` with `.unwrap()`. A type mismatch silently \
                     produces a zero/default value. Use `match` or `if let Ok(...)` instead.",
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
    fn flags_inline_try_into_val_unwrap() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, Address, Env, Symbol};
pub struct C;
#[contractimpl]
impl C {
    pub fn call_other(env: Env, contract: Address) -> i128 {
        env.invoke_contract::<i128>(&contract, &Symbol::short("get"), soroban_sdk::vec![&env])
            .try_into_val(&env)
            .unwrap()
    }
}
"#;
        let file = parse_file(src)?;
        let hits = InvokeUncheckedCastCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].severity, Severity::Medium);
        assert!(hits[0].description.contains("try_into_val"));
        Ok(())
    }

    #[test]
    fn flags_bound_result_cast_with_unwrap() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, Address, Env, Symbol, Val};
pub struct C;
#[contractimpl]
impl C {
    pub fn call_other(env: Env, contract: Address) -> i128 {
        let result = env.invoke_contract::<Val>(
            &contract, &Symbol::short("get"), soroban_sdk::vec![&env]
        );
        result.try_into_val(&env).unwrap()
    }
}
"#;
        let file = parse_file(src)?;
        let hits = InvokeUncheckedCastCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        Ok(())
    }

    #[test]
    fn no_finding_when_match_used() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, Address, Env, Symbol, Val};
pub struct C;
#[contractimpl]
impl C {
    pub fn call_other(env: Env, contract: Address) -> Option<i128> {
        let result = env.invoke_contract::<Val>(
            &contract, &Symbol::short("get"), soroban_sdk::vec![&env]
        );
        match result.try_into_val(&env) {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    }
}
"#;
        let file = parse_file(src)?;
        let hits = InvokeUncheckedCastCheck.run(&file, "");
        assert!(hits.is_empty(), "{hits:?}");
        Ok(())
    }

    #[test]
    fn no_finding_for_non_invoke_unwrap() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, Address, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn read(env: Env, key: Address) -> i128 {
        env.storage().persistent().get(&key).unwrap()
    }
}
"#;
        let file = parse_file(src)?;
        let hits = InvokeUncheckedCastCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }
}
