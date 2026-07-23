//! Detects loops over a Vec/iterable parameter where the loop body contains
//! both a storage mutation and an early-exit (return / `?` / conditional break)
//! that is reachable **after** the mutation within the same iteration.
//!
//! # Vulnerability class: batch-partial-write
//!
//! ```rust,ignore
//! for item in items.iter() {
//!     env.storage().persistent().set(&item.key, &item.value); // ← write
//!     if !validate(&item) {
//!         return Err(ContractError::Invalid); // ← exit AFTER write
//!     }
//! }
//! ```
//!
//! If `items[2]` fails validation, items 0 and 1 have already been committed
//! to storage.  Soroban's contract-local storage does not roll back on a
//! Rust-level `return Err(…)` inside the same invocation — the earlier writes
//! persist.
//!
//! # Safe two-pass pattern (suppressed)
//!
//! The check recognises the safe idiom where the function contains **two**
//! separate top-level loops over the **same** iterable: the first loop
//! contains only validation (exits, no writes) and the second only mutations
//! (writes, no exits meaningful to atomicity).  When that pattern is found,
//! no finding is emitted for either loop.
//!
//! # Algorithm
//!
//! For each `for` / `while` loop inside a `#[contractimpl]` function:
//!
//! 1. Collect all storage-mutation nodes in the loop body
//!    (`env.storage().*.set`, `.remove`, `.push_back`, `.insert`).
//! 2. Collect all early-exit nodes: `return`, `?` operator, `panic!`.
//! 3. Flag if the loop body has **at least one mutation AND at least one
//!    early-exit that appears textually after the first mutation** — a
//!    conservative linear-order approximation.
//! 4. Suppress the finding when the enclosing function has **two or more
//!    top-level loops over the same iterable** and at least one of those loops
//!    is mutation-free (pure validation) — the safe two-pass idiom.
//!
//! # Limitations
//!
//! - Statement ordering is used as a CFG approximation; a mutation inside an
//!   always-taken branch that precedes a conditional exit is not distinguished
//!   from one that may not execute.
//! - Two-pass suppression requires both loops to reference the same iterable
//!   variable name.  A rename or intermediate binding defeats it.
//! - Mutations inside helper-function calls are not tracked (false negatives).
//! - Nested loops are not recursed into; only the outermost loop level is
//!   checked.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use std::collections::{HashMap, HashSet};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ExprForLoop, ExprMethodCall, ExprReturn, ExprTry, ExprWhile, File, Stmt};

const CHECK_NAME: &str = "batch-partial-write";

pub struct BatchPartialWriteCheck;

impl Check for BatchPartialWriteCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut findings = Vec::new();

        for func in contractimpl_functions(file) {
            let fn_name = func.sig.ident.to_string();

            // Collect all top-level loops in the function body.
            let mut collector = TopLevelLoopCollector::default();
            collector.visit_block(&func.block);
            let loops = collector.loops;

            if loops.is_empty() {
                continue;
            }

            // Determine which iterables have the safe two-pass pattern.
            let safe_iterables = detect_two_pass_iterables(&loops);

            for lp in &loops {
                // Suppress if this loop is part of a confirmed two-pass pattern.
                if let Some(ref name) = lp.iter_name {
                    if safe_iterables.contains(name.as_str()) {
                        continue;
                    }
                }

                if loop_has_write_then_exit(lp) {
                    findings.push(Finding {
                        check_name: CHECK_NAME.to_string(),
                        severity: Severity::High,
                        file_path: String::new(),
                        line: lp.line,
                        function_name: fn_name.clone(),
                        description: format!(
                            "Function `{fn_name}` writes to storage inside a loop and then has \
                             an early-exit (return / ?) reachable in the same iteration. If a \
                             later element triggers the exit, earlier elements are already \
                             committed — the batch is partially applied with no rollback."
                        ),
                    });
                }
            }
        }

        findings
    }
}

// ─── Data model ──────────────────────────────────────────────────────────────

struct LoopInfo {
    /// Lowercased base variable name of the iterable (if determinable).
    iter_name: Option<String>,
    /// Source line of the loop keyword.
    line: usize,
    /// Ordered sequence of events observed in the loop body.
    events: Vec<LoopEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum LoopEvent {
    StorageWrite,
    EarlyExit,
}

// ─── Two-pass suppression ─────────────────────────────────────────────────────

/// Returns the set of iterable names for which the function uses the safe
/// two-pass idiom: at least one loop with NO mutations (validation-only) AND
/// at least one loop WITH mutations, both iterating the same variable.
fn detect_two_pass_iterables<'a>(loops: &'a [LoopInfo]) -> HashSet<&'a str> {
    let mut by_iter: HashMap<&str, (bool, bool)> = HashMap::new();
    // (has_pure_validation_loop, has_mutation_loop)
    for lp in loops {
        if let Some(ref name) = lp.iter_name {
            let has_write = lp.events.iter().any(|e| *e == LoopEvent::StorageWrite);
            let entry = by_iter.entry(name.as_str()).or_insert((false, false));
            if !has_write {
                entry.0 = true; // pure-validation loop
            } else {
                entry.1 = true; // mutation loop
            }
        }
    }

    by_iter
        .into_iter()
        .filter(|(_, (pure_val, has_mut))| *pure_val && *has_mut)
        .map(|(name, _)| name)
        .collect()
}

// ─── Vulnerable-pattern check ─────────────────────────────────────────────────

/// True when the loop body has a storage write followed (in textual order) by
/// at least one early-exit.
fn loop_has_write_then_exit(lp: &LoopInfo) -> bool {
    let mut seen_write = false;
    for event in &lp.events {
        match event {
            LoopEvent::StorageWrite => seen_write = true,
            LoopEvent::EarlyExit => {
                if seen_write {
                    return true;
                }
            }
        }
    }
    false
}

// ─── Top-level loop collector ─────────────────────────────────────────────────

/// Visits only the **direct** children of a function body and captures each
/// `for` / `while` loop without recursing into nested loops.
#[derive(Default)]
struct TopLevelLoopCollector {
    loops: Vec<LoopInfo>,
}

impl<'ast> Visit<'ast> for TopLevelLoopCollector {
    fn visit_expr_for_loop(&mut self, i: &'ast ExprForLoop) {
        let iter_name = extract_iter_name(&i.expr);
        let mut body_scan = LoopBodyScanner::default();
        body_scan.visit_block(&i.body);
        self.loops.push(LoopInfo {
            iter_name,
            line: i.for_token.span().start().line,
            events: body_scan.events,
        });
        // Do NOT recurse — nested loops belong to a different "iteration scope".
    }

    fn visit_expr_while(&mut self, i: &'ast ExprWhile) {
        let mut body_scan = LoopBodyScanner::default();
        body_scan.visit_block(&i.body);
        self.loops.push(LoopInfo {
            iter_name: None,
            line: i.while_token.span().start().line,
            events: body_scan.events,
        });
        // Do NOT recurse.
    }
}

// ─── Loop body scanner ────────────────────────────────────────────────────────

/// Walks a loop body and records `StorageWrite` and `EarlyExit` events in
/// textual (statement) order.  Does NOT descend into nested loops.
#[derive(Default)]
struct LoopBodyScanner {
    events: Vec<LoopEvent>,
}

impl<'ast> Visit<'ast> for LoopBodyScanner {
    fn visit_expr_method_call(&mut self, i: &'ast ExprMethodCall) {
        let method = i.method.to_string();
        if matches!(method.as_str(), "set" | "remove" | "push_back" | "insert") {
            if receiver_chain_has_storage(&i.receiver) {
                self.events.push(LoopEvent::StorageWrite);
            }
        }
        visit::visit_expr_method_call(self, i);
    }

    fn visit_expr_try(&mut self, i: &'ast ExprTry) {
        self.events.push(LoopEvent::EarlyExit);
        visit::visit_expr_try(self, i);
    }

    fn visit_expr_return(&mut self, i: &'ast ExprReturn) {
        self.events.push(LoopEvent::EarlyExit);
        visit::visit_expr_return(self, i);
    }

    fn visit_macro(&mut self, i: &'ast syn::Macro) {
        let name = i
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();
        if matches!(name.as_str(), "panic" | "todo" | "unreachable") {
            self.events.push(LoopEvent::EarlyExit);
        }
        visit::visit_macro(self, i);
    }

    // Do NOT descend into nested loops — their events belong to a different
    // iteration scope.
    fn visit_expr_for_loop(&mut self, _i: &'ast ExprForLoop) {}
    fn visit_expr_while(&mut self, _i: &'ast ExprWhile) {}

    fn visit_stmt(&mut self, i: &'ast Stmt) {
        visit::visit_stmt(self, i);
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Extract the base variable name from a loop's iterable expression.
/// Handles: `items`, `items.iter()`, `&items`, `items.iter().cloned()`, etc.
fn extract_iter_name(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Path(p) => p.path.get_ident().map(|i| i.to_string()),
        Expr::Reference(r) => extract_iter_name(&r.expr),
        Expr::MethodCall(m) => extract_iter_name(&m.receiver),
        _ => None,
    }
}

/// True when the receiver chain of a method call passes through `.storage()`.
fn receiver_chain_has_storage(expr: &Expr) -> bool {
    match expr {
        Expr::MethodCall(m) => {
            if m.method == "storage" {
                return true;
            }
            receiver_chain_has_storage(&m.receiver)
        }
        Expr::Field(f) => receiver_chain_has_storage(&f.base),
        _ => false,
    }
}

// ─── Unit tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Check;
    use syn::parse_file;

    // ── Vulnerable cases ──────────────────────────────────────────────────────

    #[test]
    fn flags_write_then_return_err_in_loop() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env, Vec};

pub struct C;

#[contractimpl]
impl C {
    pub fn batch_store(env: Env, items: Vec<u32>) -> Result<(), u32> {
        for item in items.iter() {
            env.storage().persistent().set(&item, &item);
            if *item == 0 {
                return Err(*item);
            }
        }
        Ok(())
    }
}
"#,
        )?;
        let hits = BatchPartialWriteCheck.run(&file, "");
        assert_eq!(hits.len(), 1, "expected one finding");
        assert_eq!(hits[0].function_name, "batch_store");
        assert_eq!(hits[0].severity, Severity::High);
        assert_eq!(hits[0].check_name, CHECK_NAME);
        Ok(())
    }

    #[test]
    fn flags_write_then_question_mark_in_loop() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env, Vec, Address};

pub struct C;

#[contractimpl]
impl C {
    pub fn apply(env: Env, recipients: Vec<Address>) -> Result<(), ()> {
        for r in recipients.iter() {
            env.storage().instance().set(&r, &true);
            r.require_auth()?;
        }
        Ok(())
    }
}
"#,
        )?;
        let hits = BatchPartialWriteCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].function_name, "apply");
        Ok(())
    }

    #[test]
    fn flags_write_then_panic_in_loop() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env, Vec};

pub struct C;

#[contractimpl]
impl C {
    pub fn process(env: Env, keys: Vec<u32>) {
        for k in keys.iter() {
            env.storage().temporary().set(&k, &k);
            if *k > 1000 {
                panic!("value too large");
            }
        }
    }
}
"#,
        )?;
        let hits = BatchPartialWriteCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].function_name, "process");
        Ok(())
    }

    #[test]
    fn flags_remove_then_return_in_loop() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env, Vec};

pub struct C;

#[contractimpl]
impl C {
    pub fn prune(env: Env, keys: Vec<u32>) -> Result<(), ()> {
        for k in keys.iter() {
            env.storage().persistent().remove(&k);
            if *k > 999 {
                return Err(());
            }
        }
        Ok(())
    }
}
"#,
        )?;
        let hits = BatchPartialWriteCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].function_name, "prune");
        Ok(())
    }

    // ── Safe cases ────────────────────────────────────────────────────────────

    #[test]
    fn passes_two_pass_validate_then_write() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env, Vec};

pub struct C;

#[contractimpl]
impl C {
    pub fn batch_store(env: Env, items: Vec<u32>) -> Result<(), u32> {
        // Pass 1: validate all elements first
        for item in items.iter() {
            if *item == 0 {
                return Err(*item);
            }
        }
        // Pass 2: write all — safe because validation passed for every element
        for item in items.iter() {
            env.storage().persistent().set(&item, &item);
        }
        Ok(())
    }
}
"#,
        )?;
        let hits = BatchPartialWriteCheck.run(&file, "");
        assert!(hits.is_empty(), "two-pass pattern should not be flagged");
        Ok(())
    }

    #[test]
    fn passes_exit_before_write_in_loop() -> Result<(), syn::Error> {
        // Validation guard comes BEFORE the write — safe.
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env, Vec};

pub struct C;

#[contractimpl]
impl C {
    pub fn guarded_store(env: Env, items: Vec<u32>) -> Result<(), u32> {
        for item in items.iter() {
            if *item == 0 {
                return Err(*item);   // exit before any write
            }
            env.storage().persistent().set(&item, &item);
        }
        Ok(())
    }
}
"#,
        )?;
        let hits = BatchPartialWriteCheck.run(&file, "");
        assert!(hits.is_empty(), "exit-before-write should not be flagged");
        Ok(())
    }

    #[test]
    fn passes_write_only_loop_no_exit() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env, Vec};

pub struct C;

#[contractimpl]
impl C {
    pub fn store_all(env: Env, items: Vec<u32>) {
        for item in items.iter() {
            env.storage().persistent().set(&item, &item);
        }
    }
}
"#,
        )?;
        let hits = BatchPartialWriteCheck.run(&file, "");
        assert!(hits.is_empty(), "write-only loop should not be flagged");
        Ok(())
    }

    #[test]
    fn passes_exit_only_loop_no_write() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env, Vec};

pub struct C;

#[contractimpl]
impl C {
    pub fn validate_all(env: Env, items: Vec<u32>) -> Result<(), u32> {
        for item in items.iter() {
            if *item == 0 {
                return Err(*item);
            }
        }
        Ok(())
    }
}
"#,
        )?;
        let hits = BatchPartialWriteCheck.run(&file, "");
        assert!(hits.is_empty(), "exit-only loop should not be flagged");
        Ok(())
    }

    #[test]
    fn passes_non_contractimpl_ignored() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{Env, Vec};

pub struct C;

impl C {
    pub fn batch_store(env: Env, items: Vec<u32>) -> Result<(), u32> {
        for item in items.iter() {
            env.storage().persistent().set(&item, &item);
            if *item == 0 { return Err(*item); }
        }
        Ok(())
    }
}
"#,
        )?;
        let hits = BatchPartialWriteCheck.run(&file, "");
        assert!(hits.is_empty(), "non-contractimpl block must be ignored");
        Ok(())
    }

    #[test]
    fn flags_second_loop_when_different_iterables() -> Result<(), syn::Error> {
        // Two-pass suppression is per-iterable; `writes` loop is still vulnerable.
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env, Vec};

pub struct C;

#[contractimpl]
impl C {
    pub fn mixed(env: Env, checks: Vec<u32>, writes: Vec<u32>) -> Result<(), u32> {
        for c in checks.iter() {
            if *c == 0 { return Err(*c); }
        }
        for w in writes.iter() {
            env.storage().persistent().set(&w, &w);
            if *w > 999 { return Err(*w); }
        }
        Ok(())
    }
}
"#,
        )?;
        let hits = BatchPartialWriteCheck.run(&file, "");
        assert_eq!(hits.len(), 1, "writes loop should still be flagged");
        assert_eq!(hits[0].function_name, "mixed");
        Ok(())
    }
}
