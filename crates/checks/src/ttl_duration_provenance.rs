//! Detects `extend_ttl` calls whose duration (`extend_to`) argument traces back to
//! a caller-controlled function parameter without any `.min()` or `.clamp()` guard.
//!
//! In Soroban:
//! - `env.storage().persistent().extend_ttl(key, threshold, extend_to)` — 3 args
//! - `env.storage().instance().extend_ttl(threshold, extend_to)` — 2 args
//!
//! If the duration argument is unbounded and caller-controlled, a caller can request
//! an extremely large TTL that either wastes storage rent indefinitely or causes the
//! call to fail/panic depending on ledger limits.

use crate::provenance::{self, BindingMap, FunctionMap, Origin};
use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use std::collections::HashMap;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ExprMethodCall, File};

const CHECK_NAME: &str = "ttl-duration-provenance";

/// Flags `extend_ttl` calls where the duration argument traces back to a function
/// parameter without `.min()` / `.clamp()` applied anywhere on the chain.
pub struct TtlDurationProvenanceCheck;

impl Check for TtlDurationProvenanceCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        let fmap = FunctionMap::from_file(file);

        for method in contractimpl_functions(file) {
            let fn_name = method.sig.ident.to_string();
            let params = provenance::collect_params(&method.sig);
            let bindings = BindingMap::from_block(&method.block);

            let mut visitor = DurationVisitor {
                fn_name,
                params,
                bindings,
                fmap: &fmap,
                out: &mut out,
            };
            visitor.visit_block(&method.block);
        }
        out
    }
}

struct DurationVisitor<'a> {
    fn_name: String,
    params: Vec<String>,
    bindings: BindingMap,
    fmap: &'a FunctionMap,
    out: &'a mut Vec<Finding>,
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

impl<'a> Visit<'_> for DurationVisitor<'a> {
    fn visit_expr_method_call(&mut self, i: &ExprMethodCall) {
        if i.method == "extend_ttl" && receiver_chain_contains_storage(&i.receiver) {
            // Determine which argument is the duration (max_ttl / extend_to):
            // - persistent/temporary: extend_ttl(key, min_ttl, max_ttl) → args[2]
            // - instance:             extend_ttl(min_ttl, max_ttl)     → args[1]
            let duration_arg = match i.args.len() {
                3 => Some(&i.args[2]),
                2 => Some(&i.args[1]),
                _ => None,
            };

            if let Some(extend_to) = duration_arg {
                let origin = provenance::trace_origin(
                    extend_to,
                    &self.params,
                    &self.bindings,
                    self.fmap,
                    &HashMap::new(),
                    provenance::MAX_HOPS,
                );

                if let Origin::Parameter(param_name) = origin {
                    self.out.push(Finding {
                        check_name: CHECK_NAME.to_string(),
                        severity: Severity::High,
                        file_path: String::new(),
                        line: i.span().start().line,
                        function_name: self.fn_name.clone(),
                        description: format!(
                            "The `extend_to` duration argument of `extend_ttl` in `{}` traces \
                             back to function parameter `{param_name}` with no `.min()` or \
                             `.clamp()` applied anywhere on the provenance chain. A caller can \
                             supply an arbitrarily large TTL value. Cap the duration with \
                             `.min(MAX_TTL)` before passing it to `extend_ttl`.",
                            self.fn_name
                        ),
                    });
                }
            }
        }
        visit::visit_expr_method_call(self, i);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Check;
    use syn::parse_file;

    fn run(src: &str) -> Vec<Finding> {
        TtlDurationProvenanceCheck.run(&parse_file(src).unwrap(), src)
    }

    #[test]
    fn flags_extend_ttl_with_uncapped_param() {
        let hits = run(r#"
use soroban_sdk::{contractimpl, Env, Symbol};
pub struct C;
#[contractimpl]
impl C {
    pub fn refresh(env: Env, requested_ttl: u32) {
        let key = Symbol::new(&env, "k");
        env.storage().persistent().extend_ttl(&key, 100, requested_ttl);
    }
}
"#);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].severity, Severity::High);
        assert_eq!(hits[0].check_name, CHECK_NAME);
        assert!(hits[0].description.contains("requested_ttl"));
    }

    #[test]
    fn passes_when_duration_literal() {
        let hits = run(r#"
use soroban_sdk::{contractimpl, Env, Symbol};
pub struct C;
#[contractimpl]
impl C {
    pub fn refresh(env: Env) {
        let key = Symbol::new(&env, "k");
        env.storage().persistent().extend_ttl(&key, 100, 200);
    }
}
"#);
        assert!(hits.is_empty());
    }

    #[test]
    fn passes_when_param_is_clamped_with_min() {
        let hits = run(r#"
use soroban_sdk::{contractimpl, Env, Symbol};
pub struct C;
#[contractimpl]
impl C {
    pub fn refresh(env: Env, requested_ttl: u32) {
        let key = Symbol::new(&env, "k");
        let capped = requested_ttl.min(10000);
        env.storage().persistent().extend_ttl(&key, 100, capped);
    }
}
"#);
        assert!(hits.is_empty(), "clamped param should pass: {hits:?}");
    }

    #[test]
    fn passes_when_param_is_clamped_with_clamp() {
        let hits = run(r#"
use soroban_sdk::{contractimpl, Env, Symbol};
pub struct C;
#[contractimpl]
impl C {
    pub fn refresh(env: Env, requested_ttl: u32) {
        let key = Symbol::new(&env, "k");
        let capped = requested_ttl.clamp(0u32, 10000);
        env.storage().persistent().extend_ttl(&key, 100, capped);
    }
}
"#);
        assert!(hits.is_empty(), "clamped param should pass: {hits:?}");
    }

    #[test]
    fn passes_when_param_min_inline() {
        let hits = run(r#"
use soroban_sdk::{contractimpl, Env, Symbol};
pub struct C;
#[contractimpl]
impl C {
    pub fn refresh(env: Env, requested_ttl: u32) {
        let key = Symbol::new(&env, "k");
        env.storage().persistent().extend_ttl(&key, 100, requested_ttl.min(10000));
    }
}
"#);
        assert!(hits.is_empty(), "inline .min() should pass: {hits:?}");
    }

    #[test]
    fn flags_indirect_through_binding() {
        let hits = run(r#"
use soroban_sdk::{contractimpl, Env, Symbol};
pub struct C;
#[contractimpl]
impl C {
    pub fn refresh(env: Env, requested_ttl: u32) {
        let key = Symbol::new(&env, "k");
        let dur = requested_ttl;
        env.storage().persistent().extend_ttl(&key, 100, dur);
    }
}
"#);
        assert_eq!(hits.len(), 1, "indirect binding to param should flag");
        assert!(hits[0].description.contains("requested_ttl"));
    }

    #[test]
    fn passes_through_helper_returning_literal() {
        let hits = run(r#"
use soroban_sdk::{contractimpl, Env, Symbol};
pub struct C;
fn safe_ttl() -> u32 { 500 }
#[contractimpl]
impl C {
    pub fn refresh(env: Env) {
        let key = Symbol::new(&env, "k");
        let dur = safe_ttl();
        env.storage().persistent().extend_ttl(&key, 100, dur);
    }
}
"#);
        assert!(
            hits.is_empty(),
            "helper returning literal should pass: {hits:?}"
        );
    }

    #[test]
    fn flags_through_helper_returning_param() {
        let hits = run(r#"
use soroban_sdk::{contractimpl, Env, Symbol};
pub struct C;
fn forward(ttl: u32) -> u32 { ttl }
#[contractimpl]
impl C {
    pub fn refresh(env: Env, requested_ttl: u32) {
        let key = Symbol::new(&env, "k");
        let dur = forward(requested_ttl);
        env.storage().persistent().extend_ttl(&key, 100, dur);
    }
}
"#);
        assert_eq!(hits.len(), 1, "helper returning param should flag");
    }

    #[test]
    fn ignores_non_contractimpl() {
        let hits = run(r#"
use soroban_sdk::{Env, Symbol};
pub struct C;
impl C {
    pub fn refresh(env: Env, requested_ttl: u32) {
        let key = Symbol::new(&env, "k");
        env.storage().persistent().extend_ttl(&key, 100, requested_ttl);
    }
}
"#);
        assert!(hits.is_empty());
    }

    #[test]
    fn passes_instance_extend_ttl_with_literal() {
        let hits = run(r#"
use soroban_sdk::{contractimpl, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn refresh(env: Env) {
        env.storage().instance().extend_ttl(100, 200);
    }
}
"#);
        assert!(hits.is_empty());
    }

    #[test]
    fn flags_instance_extend_ttl_with_param() {
        let hits = run(r#"
use soroban_sdk::{contractimpl, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn refresh(env: Env, ttl: u32) {
        env.storage().instance().extend_ttl(100, ttl);
    }
}
"#);
        assert_eq!(hits.len(), 1);
    }
}
