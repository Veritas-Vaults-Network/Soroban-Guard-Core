//! Detects admin-setting functions that do not check a revoked/blacklisted-admins
//! collection that is maintained elsewhere in the same file.
//!
//! # Vulnerability class: revoked-admin-reuse
//!
//! A contract that maintains a "removed admins" collection for audit/compliance
//! purposes has a real bug if `set_admin(new_admin)` never checks `new_admin`
//! against that collection — a previously revoked admin can be silently
//! re-appointed, defeating the revocation.
//!
//! # Algorithm (file-level, two-pass)
//!
//! **Pass 1 — detect revoked-admins collection(s):**
//! Walk every `#[contractimpl]` function body. When we find a method call of
//! `push_back` or `insert` whose receiver chain (going up to the storage key)
//! contains an identifier whose name matches a revocation keyword (revoked,
//! removed, blacklist, denylist, banned), record the storage key string (if a
//! string literal key is used) or variable name(s) involved. We also look for
//! standalone patterns: a storage `get/set` whose key identifier contains
//! revocation keywords.
//!
//! **Pass 2 — detect admin-setting functions without membership check:**
//! For each function whose name matches the ADMIN_SETTER_NAMES heuristic (set_admin,
//! set_owner, transfer_ownership, update_admin, change_admin, rotate_admin, …) that
//! also has an `Address`-typed parameter and stores something to storage: check
//! whether its body contains a `.contains(` or `.has(` call that references a
//! revocation-flavoured identifier. If no such check is present, flag it.
//!
//! # Limitations
//!
//! - Pure syntactic analysis: no type inference, no call-graph expansion.
//! - Only detects revoked-admins collections that exist in the *same* file.
//! - The membership check is identified heuristically: any `.contains(` or
//!   `.has(` call whose receiver or arguments contain a revocation-keyword
//!   identifier clears the finding.
//! - A helper function that performs the check but isn't inlined will produce a
//!   false positive.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ExprMethodCall, File, Ident, LitStr};

const CHECK_NAME: &str = "revoked-admin-reuse";

/// Function names that indicate admin rotation.
const ADMIN_SETTER_NAMES: &[&str] = &[
    "set_admin",
    "set_owner",
    "transfer_ownership",
    "update_admin",
    "change_admin",
    "rotate_admin",
    "assign_admin",
    "replace_admin",
    "new_admin",
];

/// Substrings in identifiers/keys that suggest a revocation collection.
const REVOCATION_KEYWORDS: &[&str] = &[
    "revoked",
    "removed",
    "blacklist",
    "denylist",
    "banned",
    "blocklist",
    "blocked",
];

pub struct RevokedAdminReuseCheck;

impl Check for RevokedAdminReuseCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let functions = contractimpl_functions(file);
        if functions.is_empty() {
            return vec![];
        }

        // ── Pass 1: Does this file have a revoked-admins collection? ──────────
        // Collect every revocation-flavoured identifier/key seen anywhere in the
        // contractimpl body of any function.
        let revocation_idents: Vec<String> = {
            let mut collector = RevocationCollector::default();
            for func in &functions {
                collector.visit_block(&func.block);
            }
            collector.keys
        };

        if revocation_idents.is_empty() {
            // No revoked-admins collection detected in this file — nothing to flag.
            return vec![];
        }

        // ── Pass 2: Admin-setter functions missing the membership check ───────
        let mut findings = Vec::new();

        for func in &functions {
            let fn_name = func.sig.ident.to_string();

            // Only target admin-setter functions.
            if !ADMIN_SETTER_NAMES.contains(&fn_name.as_str()) {
                // Broader heuristic: function name contains "admin" or "owner" and
                // also contains a setter verb.
                let lower = fn_name.to_lowercase();
                let is_setter = lower.contains("set_")
                    || lower.contains("update_")
                    || lower.contains("change_")
                    || lower.contains("rotate_")
                    || lower.contains("assign_")
                    || lower.contains("replace_");
                let is_admin_related = lower.contains("admin") || lower.contains("owner");
                if !(is_setter && is_admin_related) {
                    continue;
                }
            }

            // Does the function have at least one Address parameter?
            let has_address_param = func.sig.inputs.iter().any(|arg| {
                if let syn::FnArg::Typed(pt) = arg {
                    type_contains_address(&pt.ty)
                } else {
                    false
                }
            });
            if !has_address_param {
                continue;
            }

            // Does the function write to storage?
            let mut write_scan = StorageWriteScan::default();
            write_scan.visit_block(&func.block);
            if !write_scan.found_write {
                continue;
            }

            // Does the function body contain a membership check against any of
            // the revocation-flavoured identifiers?
            let mut check_scan = MembershipCheckScan {
                revocation_idents: &revocation_idents,
                found_check: false,
            };
            check_scan.visit_block(&func.block);

            if !check_scan.found_check {
                let line = func.sig.fn_token.span().start().line;
                findings.push(Finding {
                    check_name: CHECK_NAME.to_string(),
                    severity: Severity::High,
                    file_path: String::new(),
                    line,
                    function_name: fn_name.clone(),
                    description: format!(
                        "Function `{fn_name}` sets a new admin but does not check the new \
                         address against the revoked-admins collection (detected in this \
                         file). A previously revoked admin can be silently re-appointed."
                    ),
                });
            }
        }

        findings
    }
}

// ─── Helper: does a type path look like `Address`? ───────────────────────────

fn type_contains_address(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(tp) => tp
            .path
            .segments
            .iter()
            .any(|seg| seg.ident == "Address"),
        syn::Type::Reference(r) => type_contains_address(&r.elem),
        _ => false,
    }
}

// ─── Pass-1 visitor: collect revocation-keyword identifiers/keys ─────────────

/// Walks AST nodes and records every revocation-flavoured identifier seen:
/// - Idents / path segments whose name contains a revocation keyword.
/// - String literal keys (in storage `.get`/`.set` calls, etc.) whose value
///   contains a revocation keyword.
///
/// We record the *lowercased* canonical form so that the pass-2 check doesn't
/// need to match case.
#[derive(Default)]
struct RevocationCollector {
    keys: Vec<String>,
}

impl RevocationCollector {
    fn consider_str(&mut self, s: &str) {
        let lower = s.to_lowercase();
        if REVOCATION_KEYWORDS.iter().any(|kw| lower.contains(kw)) && !self.keys.contains(&lower) {
            self.keys.push(lower);
        }
    }
}

impl<'ast> Visit<'ast> for RevocationCollector {
    fn visit_ident(&mut self, i: &'ast Ident) {
        self.consider_str(&i.to_string());
        visit::visit_ident(self, i);
    }

    fn visit_lit_str(&mut self, i: &'ast LitStr) {
        self.consider_str(&i.value());
        visit::visit_lit_str(self, i);
    }
}

// ─── Pass-1 storage-write scan ───────────────────────────────────────────────

#[derive(Default)]
struct StorageWriteScan {
    found_write: bool,
}

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

impl<'ast> Visit<'ast> for StorageWriteScan {
    fn visit_expr_method_call(&mut self, i: &'ast ExprMethodCall) {
        if !self.found_write {
            let name = i.method.to_string();
            if matches!(name.as_str(), "set" | "remove" | "push_back" | "insert")
                && receiver_chain_has_storage(&i.receiver)
            {
                self.found_write = true;
            }
        }
        visit::visit_expr_method_call(self, i);
    }
}

// ─── Pass-2 membership-check scan ────────────────────────────────────────────

/// Looks for a `.contains(` or `.has(` call whose receiver or arguments
/// contain a revocation-keyword identifier, indicating the function does gate
/// on the revoked-admins list.
///
/// Also scans macro invocations (e.g. `assert!(...)`) using their token stream
/// source text, since the syn visitor does not descend into macro arguments.
struct MembershipCheckScan<'a> {
    revocation_idents: &'a [String],
    found_check: bool,
}

impl<'a> MembershipCheckScan<'a> {
    fn ident_matches_any(&self, name: &str) -> bool {
        let lower = name.to_lowercase();
        self.revocation_idents
            .iter()
            .any(|k| lower.contains(k.as_str()) || k.contains(&lower))
            || REVOCATION_KEYWORDS.iter().any(|kw| lower.contains(kw))
    }

    /// Check whether a raw token stream string (from a macro body) contains a
    /// membership-check pattern referencing a revocation-flavoured name.
    ///
    /// `proc_macro2::TokenStream::to_string()` inserts spaces between tokens,
    /// so we can't match `.contains(` literally. Instead we check that both a
    /// revocation-keyword identifier AND a membership-check method name appear
    /// anywhere in the token text — a conservative but effective heuristic.
    fn macro_tokens_have_check(&self, tokens: &str) -> bool {
        let has_membership_method = tokens.contains("contains")
            || tokens.contains("has");

        if !has_membership_method {
            return false;
        }

        // Check if any word in the tokens matches a revocation keyword.
        for word in tokens.split(|c: char| !c.is_alphanumeric() && c != '_') {
            if !word.is_empty() && self.ident_matches_any(word) {
                return true;
            }
        }
        false
    }
}

impl<'ast, 'a> Visit<'ast> for MembershipCheckScan<'a> {
    fn visit_expr_method_call(&mut self, i: &'ast ExprMethodCall) {
        if !self.found_check {
            let method_name = i.method.to_string();
            if matches!(method_name.as_str(), "contains" | "has" | "contains_key") {
                // Check if the receiver chain or any argument ident is
                // revocation-flavoured.
                let mut ident_scan = IdentCollector::default();
                ident_scan.visit_expr(&i.receiver);
                for arg in &i.args {
                    ident_scan.visit_expr(arg);
                }
                if ident_scan
                    .names
                    .iter()
                    .any(|n| self.ident_matches_any(n))
                {
                    self.found_check = true;
                }
            }
        }
        visit::visit_expr_method_call(self, i);
    }

    fn visit_macro(&mut self, i: &'ast syn::Macro) {
        if !self.found_check {
            // Macros are opaque to the AST visitor — fall back to scanning the
            // raw token stream text for membership-check patterns.
            let tokens_str = i.tokens.to_string();
            if self.macro_tokens_have_check(&tokens_str) {
                self.found_check = true;
            }
        }
        visit::visit_macro(self, i);
    }
}

/// Collects all identifiers in an expression subtree.
#[derive(Default)]
struct IdentCollector {
    names: Vec<String>,
}

impl<'ast> Visit<'ast> for IdentCollector {
    fn visit_ident(&mut self, i: &'ast Ident) {
        self.names.push(i.to_string().to_lowercase());
        visit::visit_ident(self, i);
    }
    fn visit_lit_str(&mut self, i: &'ast LitStr) {
        self.names.push(i.value().to_lowercase());
        visit::visit_lit_str(self, i);
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
    fn flags_set_admin_missing_revoked_check() -> Result<(), syn::Error> {
        // revoke_admin pushes to revoked_admins Vec; set_admin never checks it.
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Address, Env, Vec};

pub struct C;

#[contractimpl]
impl C {
    /// Revoke an existing admin and record it.
    pub fn revoke_admin(env: Env, admin: Address) {
        let mut revoked: Vec<Address> = env
            .storage()
            .persistent()
            .get(&"revoked_admins")
            .unwrap_or_default();
        revoked.push_back(admin.clone());
        env.storage().persistent().set(&"revoked_admins", &revoked);
    }

    /// Set a new admin — BUG: never checks revoked_admins.
    pub fn set_admin(env: Env, new_admin: Address) {
        env.require_auth();
        env.storage().persistent().set(&"admin", &new_admin);
    }
}
"#,
        )?;
        let hits = RevokedAdminReuseCheck.run(&file, "");
        assert_eq!(hits.len(), 1, "expected one finding");
        assert_eq!(hits[0].function_name, "set_admin");
        assert_eq!(hits[0].severity, Severity::High);
        assert_eq!(hits[0].check_name, CHECK_NAME);
        Ok(())
    }

    #[test]
    fn flags_set_owner_missing_blacklist_check() -> Result<(), syn::Error> {
        // Uses the word "blacklist" in variable name.
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Address, Env, Vec};

pub struct C;

#[contractimpl]
impl C {
    pub fn ban_admin(env: Env, addr: Address) {
        let mut blacklist: Vec<Address> = env
            .storage()
            .instance()
            .get(&"blacklist")
            .unwrap_or_default();
        blacklist.push_back(addr);
        env.storage().instance().set(&"blacklist", &blacklist);
    }

    pub fn set_owner(env: Env, new_owner: Address) {
        env.require_auth();
        // Missing: check blacklist before accepting new_owner
        env.storage().instance().set(&"owner", &new_owner);
    }
}
"#,
        )?;
        let hits = RevokedAdminReuseCheck.run(&file, "");
        assert_eq!(hits.len(), 1, "expected one finding for set_owner");
        assert_eq!(hits[0].function_name, "set_owner");
        Ok(())
    }

    #[test]
    fn flags_rotate_admin_missing_removed_check() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Address, Env, Map};

pub struct C;

#[contractimpl]
impl C {
    pub fn remove_admin(env: Env, addr: Address) {
        let mut removed: Map<Address, bool> = env
            .storage()
            .persistent()
            .get(&"removed_admins")
            .unwrap_or_default();
        removed.set(addr.clone(), true);
        env.storage().persistent().set(&"removed_admins", &removed);
    }

    pub fn rotate_admin(env: Env, new_admin: Address) {
        env.require_auth();
        // Bug: no check against removed_admins
        env.storage().persistent().set(&"current_admin", &new_admin);
    }
}
"#,
        )?;
        let hits = RevokedAdminReuseCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].function_name, "rotate_admin");
        Ok(())
    }

    // ── Safe cases ────────────────────────────────────────────────────────────

    #[test]
    fn passes_set_admin_with_contains_check() -> Result<(), syn::Error> {
        // set_admin explicitly checks revoked_admins.contains(new_admin)
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Address, Env, Vec};

pub struct C;

#[contractimpl]
impl C {
    pub fn revoke_admin(env: Env, admin: Address) {
        let mut revoked: Vec<Address> = env
            .storage()
            .persistent()
            .get(&"revoked_admins")
            .unwrap_or_default();
        revoked.push_back(admin.clone());
        env.storage().persistent().set(&"revoked_admins", &revoked);
    }

    pub fn set_admin(env: Env, new_admin: Address) {
        env.require_auth();
        let revoked: Vec<Address> = env
            .storage()
            .persistent()
            .get(&"revoked_admins")
            .unwrap_or_default();
        assert!(!revoked.contains(&new_admin), "admin was revoked");
        env.storage().persistent().set(&"admin", &new_admin);
    }
}
"#,
        )?;
        let hits = RevokedAdminReuseCheck.run(&file, "");
        assert!(hits.is_empty(), "expected no findings when check is present");
        Ok(())
    }

    #[test]
    fn passes_set_owner_with_has_on_blacklist_storage() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Address, Env};

pub struct C;

#[contractimpl]
impl C {
    pub fn ban_address(env: Env, addr: Address) {
        env.storage().persistent().set(&"blacklist", &addr);
    }

    pub fn set_owner(env: Env, new_owner: Address) {
        env.require_auth();
        let is_blacklisted = env.storage().persistent().has(&"blacklist");
        assert!(!is_blacklisted, "owner is blacklisted");
        env.storage().instance().set(&"owner", &new_owner);
    }
}
"#,
        )?;
        let hits = RevokedAdminReuseCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }

    #[test]
    fn passes_when_no_revocation_collection_exists() -> Result<(), syn::Error> {
        // Contract has set_admin but no revocation collection at all — not flagged.
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Address, Env};

pub struct C;

#[contractimpl]
impl C {
    pub fn set_admin(env: Env, new_admin: Address) {
        env.require_auth();
        env.storage().persistent().set(&"admin", &new_admin);
    }
}
"#,
        )?;
        let hits = RevokedAdminReuseCheck.run(&file, "");
        assert!(
            hits.is_empty(),
            "should not flag when no revocation collection exists"
        );
        Ok(())
    }

    #[test]
    fn ignores_non_contractimpl_impl() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{Address, Env, Vec};

pub struct C;

impl C {
    pub fn revoke_admin(env: Env, admin: Address) {
        let mut revoked: Vec<Address> = env
            .storage()
            .persistent()
            .get(&"revoked_admins")
            .unwrap_or_default();
        revoked.push_back(admin);
        env.storage().persistent().set(&"revoked_admins", &revoked);
    }

    pub fn set_admin(env: Env, new_admin: Address) {
        env.require_auth();
        env.storage().persistent().set(&"admin", &new_admin);
    }
}
"#,
        )?;
        let hits = RevokedAdminReuseCheck.run(&file, "");
        assert!(
            hits.is_empty(),
            "should only analyze #[contractimpl] blocks"
        );
        Ok(())
    }

    #[test]
    fn passes_rotate_admin_with_denylist_contains() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Address, Env, Vec};

pub struct C;

#[contractimpl]
impl C {
    pub fn add_to_denylist(env: Env, addr: Address) {
        let mut denylist: Vec<Address> = env
            .storage()
            .instance()
            .get(&"denylist")
            .unwrap_or_default();
        denylist.push_back(addr);
        env.storage().instance().set(&"denylist", &denylist);
    }

    pub fn rotate_admin(env: Env, new_admin: Address) {
        env.require_auth();
        let denylist: Vec<Address> = env
            .storage()
            .instance()
            .get(&"denylist")
            .unwrap_or_default();
        assert!(!denylist.contains(&new_admin));
        env.storage().instance().set(&"admin", &new_admin);
    }
}
"#,
        )?;
        let hits = RevokedAdminReuseCheck.run(&file, "");
        assert!(hits.is_empty(), "rotate_admin does check the denylist");
        Ok(())
    }

    #[test]
    fn flags_update_admin_heuristic_name() -> Result<(), syn::Error> {
        // Not in ADMIN_SETTER_NAMES but matches broader heuristic.
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Address, Env, Vec};

pub struct C;

#[contractimpl]
impl C {
    pub fn record_revoked(env: Env, addr: Address) {
        let mut revoked: Vec<Address> = env
            .storage()
            .persistent()
            .get(&"revoked_admins")
            .unwrap_or_default();
        revoked.push_back(addr);
        env.storage().persistent().set(&"revoked_admins", &revoked);
    }

    pub fn update_admin(env: Env, new_admin: Address) {
        env.require_auth();
        env.storage().persistent().set(&"admin", &new_admin);
    }
}
"#,
        )?;
        let hits = RevokedAdminReuseCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].function_name, "update_admin");
        Ok(())
    }
}
