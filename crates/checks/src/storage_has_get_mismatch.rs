//! Detects storage key check/use mismatches: `has(key_a)` guarding a block that
//! later reads a *different* key via `get(key_b)` on the same storage tier.
//!
//! In Soroban, `has(k)` establishes that `k` exists on that tier. A `get` of a
//! different key inside the guarded branch is not covered by that existence
//! check, so it can silently return `None`/default data (a check/use TOCTOU).
//!
//! Guards are scoped to the `if` branch whose condition they appear in: a
//! `get` is only compared against the `has` guards that are lexically in scope
//! at that point, so independent `has(A)/get(A)` then `has(B)/get(B)` pairs in
//! the same function are not cross-flagged.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{BinOp, Expr, ExprIf, ExprMethodCall, File};

const CHECK_NAME: &str = "storage-has-get-mismatch";

/// An active `has(key)` guard on a storage tier, scoped to the `if` branch whose
/// condition established it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Guard {
    tier: String,
    key: String,
    line: usize,
}

/// Flags `has(key_a)` guards followed by `get(key_b)` on the same storage tier
/// where the keys differ and the `get` falls inside the guarded branch.
pub struct StorageHasGetMismatchCheck;

impl Check for StorageHasGetMismatchCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        for method in contractimpl_functions(file) {
            let fn_name = method.sig.ident.to_string();
            let mut v = StorageVisitor {
                fn_name: fn_name.clone(),
                out: &mut out,
                guards: Vec::new(),
            };
            v.visit_block(&method.block);
        }
        out
    }
}

fn get_storage_tier(m: &ExprMethodCall) -> Option<String> {
    let mut current = &m.receiver;
    loop {
        match &**current {
            Expr::MethodCall(mc) => {
                if matches!(
                    mc.method.to_string().as_str(),
                    "persistent" | "instance" | "temporary"
                ) {
                    return Some(mc.method.to_string());
                }
                current = &mc.receiver;
            }
            _ => return None,
        }
    }
}

/// Token-level representation of the first argument, with a leading `&` reference
/// stripped so `&KEY` and `KEY` (and `&"key"` / `"key"`) compare equal and render
/// readably in findings.
fn first_arg_tokens(m: &ExprMethodCall) -> Option<String> {
    let arg = m.args.first()?;
    match arg {
        Expr::Reference(r) => Some(r.expr.to_token_stream().to_string()),
        other => Some(other.to_token_stream().to_string()),
    }
}

/// Collect the `has` guards that are *certainly* true when the `if` branch runs.
///
/// Walks through `&&` chains and parentheses, but stops at `||`, `!`, comparisons
/// and any other non-short-circuit-true position, where a nested `has` may not be
/// the reason the branch was taken.
fn collect_cond_guards(expr: &Expr, out: &mut Vec<Guard>) {
    match expr {
        Expr::Binary(b) if matches!(b.op, BinOp::And(_)) => {
            collect_cond_guards(&b.left, out);
            collect_cond_guards(&b.right, out);
        }
        Expr::Paren(p) => collect_cond_guards(&p.expr, out),
        Expr::MethodCall(m) if m.method == "has" => {
            if let (Some(tier), Some(key)) = (get_storage_tier(m), first_arg_tokens(m)) {
                out.push(Guard {
                    tier,
                    key,
                    line: m.span().start().line,
                });
            }
        }
        _ => {}
    }
}

struct StorageVisitor<'a> {
    fn_name: String,
    out: &'a mut Vec<Finding>,
    guards: Vec<Guard>,
}

impl<'ast> Visit<'ast> for StorageVisitor<'_> {
    fn visit_expr_if(&mut self, i: &'ast ExprIf) {
        // Deliberately do not call the default visitor here: it walks the
        // condition and both branches with no guard scoping. Instead we push the
        // `has` guards established by the condition before visiting the `then`
        // branch and pop them afterwards, so inner `get`s can only be attributed
        // to `has` checks that lexically enclose them.

        // The condition itself may contain storage reads; visit it unguarded.
        self.visit_expr(&i.cond);

        let mut cond_guards = Vec::new();
        collect_cond_guards(&i.cond, &mut cond_guards);

        let saved_len = self.guards.len();
        for guard in cond_guards {
            if !self.guards.contains(&guard) {
                self.guards.push(guard);
            }
        }
        self.visit_block(&i.then_branch);
        self.guards.truncate(saved_len);

        if let Some((_, else_expr)) = &i.else_branch {
            self.visit_expr(else_expr);
        }
    }

    fn visit_expr_method_call(&mut self, i: &'ast ExprMethodCall) {
        if i.method == "get" {
            if let (Some(tier), Some(key)) = (get_storage_tier(i), first_arg_tokens(i)) {
                let line = i.span().start().line;
                // Only flag when a same-tier guard is in scope but none of them
                // covers this key — reading a key that is genuinely guarded by an
                // outer `has` is fine (e.g. a nested `has(B)` inside `has(A)`).
                let mut in_scope = false;
                let mut covered = false;
                let mut guard_line = 0;
                let mut has_key = String::new();
                for guard in &self.guards {
                    if guard.tier == tier {
                        in_scope = true;
                        guard_line = guard.line;
                        has_key = guard.key.clone();
                        if guard.key == key {
                            covered = true;
                        }
                    }
                }
                if in_scope && !covered {
                    self.out.push(Finding {
                        check_name: CHECK_NAME.to_string(),
                        severity: Severity::Medium,
                        file_path: String::new(),
                        line,
                        function_name: self.fn_name.clone(),
                        description: format!(
                            "Mismatch in `{tier}` storage in `{}`: the block is guarded by \
                             has({has_key}) at line {has_line} but reads get({key}) at line \
                             {line}. The existence check does not cover this read, so \
                             get({key}) may silently return `None`/default data when the key \
                             is absent. Check the same key that you read, or guard the read \
                             with has({key}).",
                            self.fn_name,
                            has_key = has_key,
                            has_line = guard_line,
                            key = key,
                            line = line,
                            tier = tier
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

    fn run_on_src(src: &str) -> Result<Vec<Finding>, syn::Error> {
        let file = parse_file(src)?;
        Ok(StorageHasGetMismatchCheck.run(&file, src))
    }

    #[test]
    fn flags_has_get_mismatch() -> Result<(), syn::Error> {
        let hits = run_on_src(
            r#"
use soroban_sdk::{contractimpl, symbol_short, Env};

pub struct C;

const K1: soroban_sdk::Symbol = symbol_short!("k1");
const K2: soroban_sdk::Symbol = symbol_short!("k2");

#[contractimpl]
impl C {
    pub fn process(env: Env) {
        env.require_auth();
        if env.storage().persistent().has(&K1) {
            let val = env.storage().persistent().get(&K2);
        }
    }
}
"#,
        )?;
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].severity, Severity::Medium);
        assert!(hits[0].description.contains("Mismatch"));
        Ok(())
    }

    #[test]
    fn passes_matching_keys() -> Result<(), syn::Error> {
        let hits = run_on_src(
            r#"
use soroban_sdk::{contractimpl, symbol_short, Env};

pub struct C;

const K: soroban_sdk::Symbol = symbol_short!("k");

#[contractimpl]
impl C {
    pub fn process(env: Env) {
        env.require_auth();
        if env.storage().persistent().has(&K) {
            let val = env.storage().persistent().get(&K);
        }
    }
}
"#,
        )?;
        assert!(hits.is_empty());
        Ok(())
    }

    #[test]
    fn passes_different_tiers() -> Result<(), syn::Error> {
        let hits = run_on_src(
            r#"
use soroban_sdk::{contractimpl, symbol_short, Env};

pub struct C;

const K1: soroban_sdk::Symbol = symbol_short!("k1");
const K2: soroban_sdk::Symbol = symbol_short!("k2");

#[contractimpl]
impl C {
    pub fn process(env: Env) {
        env.require_auth();
        if env.storage().persistent().has(&K1) {
            let val = env.storage().instance().get(&K2);
        }
    }
}
"#,
        )?;
        assert!(hits.is_empty());
        Ok(())
    }

    #[test]
    fn passes_sequential_guards_on_same_tier() -> Result<(), syn::Error> {
        // Two independent has/get pairs on the same tier must not be
        // cross-flagged: each get is guarded by its own has.
        let hits = run_on_src(
            r#"
use soroban_sdk::{contractimpl, symbol_short, Env};

pub struct C;

const K1: soroban_sdk::Symbol = symbol_short!("k1");
const K2: soroban_sdk::Symbol = symbol_short!("k2");

#[contractimpl]
impl C {
    pub fn process(env: Env) {
        env.require_auth();
        if env.storage().persistent().has(&K1) {
            let v1 = env.storage().persistent().get(&K1);
            if env.storage().persistent().has(&K2) {
                let v2 = env.storage().persistent().get(&K2);
                let _ = (v1, v2);
            }
        }
    }
}
"#,
        )?;
        assert!(hits.is_empty(), "unexpected findings: {hits:#?}");
        Ok(())
    }

    #[test]
    fn passes_get_covered_by_outer_guard() -> Result<(), syn::Error> {
        // A get keyed on an outer guard is legitimate even when an inner guard
        // on a different key is in scope.
        let hits = run_on_src(
            r#"
use soroban_sdk::{contractimpl, symbol_short, Env};

pub struct C;

const K1: soroban_sdk::Symbol = symbol_short!("k1");
const K2: soroban_sdk::Symbol = symbol_short!("k2");

#[contractimpl]
impl C {
    pub fn process(env: Env) {
        env.require_auth();
        if env.storage().persistent().has(&K1) {
            if env.storage().persistent().has(&K2) {
                let v1 = env.storage().persistent().get(&K1);
                let v2 = env.storage().persistent().get(&K2);
                let _ = (v1, v2);
            }
        }
    }
}
"#,
        )?;
        assert!(hits.is_empty(), "unexpected findings: {hits:#?}");
        Ok(())
    }

    #[test]
    fn passes_get_outside_guard_scope() -> Result<(), syn::Error> {
        // A get after the guarded branch has closed is not attributed to the
        // earlier has.
        let hits = run_on_src(
            r#"
use soroban_sdk::{contractimpl, symbol_short, Env};

pub struct C;

const K1: soroban_sdk::Symbol = symbol_short!("k1");
const K2: soroban_sdk::Symbol = symbol_short!("k2");

#[contractimpl]
impl C {
    pub fn process(env: Env) {
        env.require_auth();
        if env.storage().persistent().has(&K1) {
            let _v = env.storage().persistent().get(&K1);
        }
        let other = env.storage().persistent().get(&K2);
        let _ = other;
    }
}
"#,
        )?;
        assert!(hits.is_empty(), "unexpected findings: {hits:#?}");
        Ok(())
    }

    #[test]
    fn passes_get_in_else_branch_unattributed_to_guard() -> Result<(), syn::Error> {
        // A get in an else branch is not covered by the has guard (branch runs
        // when the key is absent), so it must not be flagged as a mismatch with
        // the has key.
        let hits = run_on_src(
            r#"
use soroban_sdk::{contractimpl, symbol_short, Env};

pub struct C;

const K1: soroban_sdk::Symbol = symbol_short!("k1");
const K2: soroban_sdk::Symbol = symbol_short!("k2");

#[contractimpl]
impl C {
    pub fn process(env: Env) {
        env.require_auth();
        if env.storage().persistent().has(&K1) {
            let _v = env.storage().persistent().get(&K1);
        } else {
            let _v = env.storage().persistent().get(&K2);
        }
    }
}
"#,
        )?;
        assert!(hits.is_empty(), "unexpected findings: {hits:#?}");
        Ok(())
    }

    #[test]
    fn flags_nested_mismatch_on_second_get() -> Result<(), syn::Error> {
        // Inside a has-guarded branch, an additional get of a different key is
        // not covered by any in-scope guard.
        let hits = run_on_src(
            r#"
use soroban_sdk::{contractimpl, symbol_short, Env};

pub struct C;

const K1: soroban_sdk::Symbol = symbol_short!("k1");
const K2: soroban_sdk::Symbol = symbol_short!("k2");

#[contractimpl]
impl C {
    pub fn process(env: Env) {
        env.require_auth();
        if env.storage().persistent().has(&K1) {
            let _v1 = env.storage().persistent().get(&K1);
            let _v2 = env.storage().persistent().get(&K2);
        }
    }
}
"#,
        )?;
        assert_eq!(hits.len(), 1, "got {hits:#?}");
        Ok(())
    }
}
