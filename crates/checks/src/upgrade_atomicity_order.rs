//! Ordering-correct version of `upgrade_no_schema_version`: the version/schema
//! write must happen-before `update_current_contract_wasm` on *every* path
//! that reaches the upgrade call, not merely exist somewhere in the file.
//!
//! `upgrade_no_schema_version.rs` is a whole-file substring check: it passes
//! as soon as *any* function anywhere writes a key whose tokens contain
//! "version"/"schema", regardless of whether that write can actually run
//! before the upgrade call on the path that reaches it. That means a write
//! sitting inside an `if` branch that doesn't cover every path to the upgrade
//! call is invisible to it — the write "exists in the file", so the old
//! check is satisfied, even though half the entrypoint's control-flow paths
//! swap the WASM with no version bump at all.
//!
//! This check instead builds the CFG (see `crate::cfg`) for each
//! `#[contractimpl]` function, inlining calls to other functions in the file
//! so a write or the upgrade call hidden behind a helper still participates,
//! and asks a dominance question: does the block containing the version/
//! schema write dominate the block containing the `update_current_contract_wasm`
//! call, with the write occurring first when they share a block? If not, the
//! upgrade call is flagged, even when a schema-version write exists
//! *somewhere* in the function.

use crate::cfg;
use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use proc_macro2::TokenStream;
use quote::ToTokens;
use std::collections::HashMap;
use syn::{Block, Expr, File, ImplItem, Item};

const CHECK_NAME: &str = "upgrade-atomicity-order";

const TAG_VERSION_WRITE: u8 = 0;
const TAG_UPGRADE_CALL: u8 = 1;

/// Flags `update_current_contract_wasm` calls that are reachable on some
/// control-flow path without a version/schema storage write happening first.
pub struct UpgradeAtomicityOrderCheck;

impl Check for UpgradeAtomicityOrderCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let registry = collect_fn_registry(file);
        let classify: &dyn Fn(&Expr) -> Option<u8> = &classify_expr;

        let mut out = Vec::new();
        for method in contractimpl_functions(file) {
            let fn_name = method.sig.ident.to_string();
            let graph = cfg::build(&method.block, &registry, classify);

            let upgrade_markers = graph.markers_with_tag(TAG_UPGRADE_CALL);
            if upgrade_markers.is_empty() {
                continue;
            }
            let write_markers = graph.markers_with_tag(TAG_VERSION_WRITE);

            for (u_block, u_marker) in &upgrade_markers {
                let dominated = write_markers
                    .iter()
                    .any(|(w_block, w_marker)| {
                        graph.marker_dominates(*w_block, *w_marker, *u_block, *u_marker)
                    });
                if !dominated {
                    out.push(Finding {
                        check_name: CHECK_NAME.to_string(),
                        severity: Severity::High,
                        file_path: String::new(),
                        line: u_marker.line,
                        function_name: fn_name.clone(),
                        description: format!(
                            "Function `{fn_name}` calls `update_current_contract_wasm` on a \
                             control-flow path that does not unconditionally pass through a \
                             version/schema storage write first. A version/schema key may exist \
                             elsewhere in the file, but this specific path to the WASM swap can \
                             execute without writing it, leaving upgraded code unable to detect \
                             the storage layout it is reading."
                        ),
                    });
                }
            }
        }
        out
    }
}

fn classify_expr(expr: &Expr) -> Option<u8> {
    let Expr::MethodCall(m) = expr else {
        return None;
    };
    if m.method == "update_current_contract_wasm" {
        return Some(TAG_UPGRADE_CALL);
    }
    if m.method == "set" && receiver_chain_contains_storage(&m.receiver) {
        if let Some(key_arg) = m.args.first() {
            if key_tokens_contain_version(key_arg) {
                return Some(TAG_VERSION_WRITE);
            }
        }
    }
    None
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

fn key_tokens_contain_version(expr: &Expr) -> bool {
    let mut ts = TokenStream::new();
    expr.to_tokens(&mut ts);
    let s = ts.to_string().to_lowercase();
    s.contains("version") || s.contains("schema")
}

/// Every named function body in the file — `#[contractimpl]` methods, methods
/// in any other `impl` block, and free functions — so a call to a private
/// helper resolves to something the CFG builder can inline.
fn collect_fn_registry(file: &File) -> HashMap<String, &Block> {
    let mut registry = HashMap::new();
    for item in &file.items {
        match item {
            Item::Impl(item_impl) => {
                for impl_item in &item_impl.items {
                    if let ImplItem::Fn(f) = impl_item {
                        registry.insert(f.sig.ident.to_string(), &f.block);
                    }
                }
            }
            Item::Fn(f) => {
                registry.insert(f.sig.ident.to_string(), &f.block);
            }
            _ => {}
        }
    }
    registry
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Check;
    use syn::parse_file;

    fn run(src: &str) -> Vec<Finding> {
        let file = parse_file(src).unwrap();
        UpgradeAtomicityOrderCheck.run(&file, src)
    }

    #[test]
    fn passes_when_write_unconditionally_precedes_upgrade() {
        let hits = run(
            r#"
use soroban_sdk::{contract, contractimpl, symbol_short, BytesN, Env};

#[contract]
pub struct C;

const SCHEMA_VERSION: soroban_sdk::Symbol = symbol_short!("ver");

#[contractimpl]
impl C {
    pub fn upgrade(env: Env, wasm_hash: BytesN<32>) {
        env.storage().instance().set(&SCHEMA_VERSION, &2u32);
        env.deployer().update_current_contract_wasm(wasm_hash);
    }
}
"#,
        );
        assert!(hits.is_empty());
    }

    #[test]
    fn flags_write_inside_if_branch_that_skips_to_upgrade() {
        let hits = run(
            r#"
use soroban_sdk::{contract, contractimpl, symbol_short, Address, BytesN, Env};

#[contract]
pub struct C;

const SCHEMA_VERSION: soroban_sdk::Symbol = symbol_short!("ver");

#[contractimpl]
impl C {
    pub fn upgrade(env: Env, wasm_hash: BytesN<32>, bump_schema: bool) {
        if bump_schema {
            env.storage().instance().set(&SCHEMA_VERSION, &2u32);
        }
        env.deployer().update_current_contract_wasm(wasm_hash);
    }
}
"#,
        );
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].check_name, CHECK_NAME);
        assert_eq!(hits[0].severity, Severity::High);
        assert_eq!(hits[0].function_name, "upgrade");
    }

    #[test]
    fn flags_write_after_the_upgrade_call() {
        let hits = run(
            r#"
use soroban_sdk::{contract, contractimpl, symbol_short, BytesN, Env};

#[contract]
pub struct C;

const SCHEMA_VERSION: soroban_sdk::Symbol = symbol_short!("ver");

#[contractimpl]
impl C {
    pub fn upgrade(env: Env, wasm_hash: BytesN<32>) {
        env.deployer().update_current_contract_wasm(wasm_hash);
        env.storage().instance().set(&SCHEMA_VERSION, &2u32);
    }
}
"#,
        );
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn passes_when_write_precedes_upgrade_in_every_branch() {
        let hits = run(
            r#"
use soroban_sdk::{contract, contractimpl, symbol_short, BytesN, Env};

#[contract]
pub struct C;

const SCHEMA_VERSION: soroban_sdk::Symbol = symbol_short!("ver");

#[contractimpl]
impl C {
    pub fn upgrade(env: Env, wasm_hash: BytesN<32>, other_hash: BytesN<32>, use_other: bool) {
        env.storage().instance().set(&SCHEMA_VERSION, &2u32);
        if use_other {
            env.deployer().update_current_contract_wasm(other_hash);
        } else {
            env.deployer().update_current_contract_wasm(wasm_hash);
        }
    }
}
"#,
        );
        assert!(hits.is_empty());
    }

    #[test]
    fn flags_when_upgrade_reachable_without_write_via_a_helper() {
        let hits = run(
            r#"
use soroban_sdk::{contract, contractimpl, symbol_short, BytesN, Env};

#[contract]
pub struct C;

const SCHEMA_VERSION: soroban_sdk::Symbol = symbol_short!("ver");

#[contractimpl]
impl C {
    pub fn upgrade(env: Env, wasm_hash: BytesN<32>, bump_schema: bool) {
        Self::maybe_bump(env.clone(), bump_schema);
        env.deployer().update_current_contract_wasm(wasm_hash);
    }

    fn maybe_bump(env: Env, bump_schema: bool) {
        if bump_schema {
            env.storage().instance().set(&SCHEMA_VERSION, &2u32);
        }
    }
}
"#,
        );
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].function_name, "upgrade");
    }

    #[test]
    fn passes_when_helper_unconditionally_writes_before_upgrade() {
        let hits = run(
            r#"
use soroban_sdk::{contract, contractimpl, symbol_short, BytesN, Env};

#[contract]
pub struct C;

const SCHEMA_VERSION: soroban_sdk::Symbol = symbol_short!("ver");

#[contractimpl]
impl C {
    pub fn upgrade(env: Env, wasm_hash: BytesN<32>) {
        Self::bump(env.clone());
        env.deployer().update_current_contract_wasm(wasm_hash);
    }

    fn bump(env: Env) {
        env.storage().instance().set(&SCHEMA_VERSION, &2u32);
    }
}
"#,
        );
        assert!(hits.is_empty());
    }

    #[test]
    fn no_findings_when_no_upgrade_call() {
        let hits = run(
            r#"
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct C;

#[contractimpl]
impl C {
    pub fn init(env: Env) {
        env.storage().instance().set(&42u32, &0u32);
    }
}
"#,
        );
        assert!(hits.is_empty());
    }

    #[test]
    fn flags_when_write_only_reachable_via_a_different_branch_than_upgrade() {
        let hits = run(
            r#"
use soroban_sdk::{contract, contractimpl, symbol_short, BytesN, Env};

#[contract]
pub struct C;

const SCHEMA_VERSION: soroban_sdk::Symbol = symbol_short!("ver");

#[contractimpl]
impl C {
    pub fn upgrade(env: Env, wasm_hash: BytesN<32>, legacy: bool) {
        if legacy {
            env.storage().instance().set(&SCHEMA_VERSION, &2u32);
        } else {
            env.deployer().update_current_contract_wasm(wasm_hash);
        }
    }
}
"#,
        );
        assert_eq!(hits.len(), 1);
    }
}
