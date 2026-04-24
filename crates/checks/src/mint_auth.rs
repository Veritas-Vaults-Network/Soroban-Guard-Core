//! Flags `pub fn mint` in `#[contractimpl]` blocks where `require_auth()` is called
//! on a recipient parameter (`to` or `recipient`) instead of on a privileged
//! admin/minter address loaded from contract storage.
//!
//! ## Vulnerability
//!
//! A mint function that does `to.require_auth()` lets *any* user mint tokens to
//! themselves  they simply sign their own transaction. The auth check must be on a
//! minter/admin address stored in contract state, not on the recipient.
//!
//! ## Detection
//!
//! For every `pub fn mint` inside a `#[contractimpl]` block:
//!   1. Collect all function parameter names.
//!   2. Walk the body for `.require_auth()` calls.
//!   3. Flag if any `require_auth()` receiver is a direct parameter named `to` or
//!      `recipient` AND no `require_auth()` receiver is derived from a storage `.get`
//!      call (the safe pattern).

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ExprMethodCall, FnArg, File, Pat, PatType, Visibility};

const CHECK_NAME: &str = "mint-auth-on-recipient";

/// Recipient-like parameter names that should never be the auth subject in `mint`.
const RECIPIENT_PARAMS: &[&str] = &["to", "recipient"];

pub struct MintAuthCheck;

impl Check for MintAuthCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();

        for method in contractimpl_functions(file) {
            // Only care about public `mint` functions.
            if !matches!(method.vis, Visibility::Public(_)) {
                continue;
            }
            if method.sig.ident != "mint" {
                continue;
            }

            // Collect all parameter names for this function.
            let param_names: Vec<String> = method
                .sig
                .inputs
                .iter()
                .filter_map(|arg| {
                    if let FnArg::Typed(PatType { pat, .. }) = arg {
                        if let Pat::Ident(pi) = pat.as_ref() {
                            return Some(pi.ident.to_string());
                        }
                    }
                    None
                })
                .collect();

            let mut scan = AuthScan::default();
            scan.visit_block(&method.block);

            // Determine if any require_auth receiver is a recipient param.
            let auth_on_recipient = scan.require_auth_receivers.iter().any(|recv| {
                RECIPIENT_PARAMS.contains(&recv.as_str()) && param_names.contains(recv)
            });

            // Determine if any require_auth receiver comes from storage (safe pattern).
            let auth_on_storage_value = scan.has_storage_backed_auth;

            if auth_on_recipient && !auth_on_storage_value {
                let line = method.sig.fn_token.span().start().line;
                let fn_name = method.sig.ident.to_string();
                out.push(Finding {
                    check_name: CHECK_NAME.to_string(),
                    severity: Severity::High,
                    file_path: String::new(),
                    line,
                    function_name: fn_name.clone(),
                    description: format!(
                        "Function `{fn_name}` calls `require_auth()` on the recipient \
                         parameter (`to`/`recipient`) instead of on a privileged \
                         admin/minter address loaded from storage. Any user can authorize \
                         themselves as the recipient and mint tokens freely."
                    ),
                });
            }
        }

        out
    }
}

/// Walks a function body and collects:
/// - Names of identifiers on which `.require_auth()` is called directly.
/// - Whether any `.require_auth()` call is on a value obtained from a storage `.get(...)`.
#[derive(Default)]
struct AuthScan {
    /// Ident names that appear as the direct receiver of `.require_auth()`.
    require_auth_receivers: Vec<String>,
    /// True if at least one `.require_auth()` receiver is a storage-read value
    /// (i.e., the receiver chain contains a `.get(...)` on `env.storage()`).
    has_storage_backed_auth: bool,
}

impl<'ast> Visit<'ast> for AuthScan {
    fn visit_expr_method_call(&mut self, i: &'ast ExprMethodCall) {
        if i.method == "require_auth" {
            match &*i.receiver {
                // Direct ident: `to.require_auth()`, `admin.require_auth()`, etc.
                Expr::Path(p) => {
                    if let Some(ident) = p.path.get_ident() {
                        self.require_auth_receivers.push(ident.to_string());
                    }
                }
                // Method-call chain: could be a storage read result used inline,
                // e.g. `env.storage().instance().get(&KEY).unwrap().require_auth()`
                recv => {
                    if receiver_chain_has_storage_get(recv) {
                        self.has_storage_backed_auth = true;
                    }
                }
            }
        }
        visit::visit_expr_method_call(self, i);
    }
}

/// Returns true if the expression chain contains a `.get(...)` call whose receiver
/// chain includes `.storage()`  indicating the value came from contract storage.
fn receiver_chain_has_storage_get(expr: &Expr) -> bool {
    match expr {
        Expr::MethodCall(m) => {
            if m.method == "get" && receiver_chain_has_storage(expr) {
                return true;
            }
            receiver_chain_has_storage_get(&m.receiver)
        }
        _ => false,
    }
}

/// Returns true if the expression chain contains a `.storage()` call.
fn receiver_chain_has_storage(expr: &Expr) -> bool {
    match expr {
        Expr::MethodCall(m) => {
            if m.method == "storage" {
                return true;
            }
            receiver_chain_has_storage(&m.receiver)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Check;
    use syn::parse_file;

    fn run(src: &str) -> Vec<Finding> {
        MintAuthCheck.run(&parse_file(src).unwrap(), src)
    }

    //  Vulnerable cases 

    #[test]
    fn flags_mint_with_to_require_auth() {
        let hits = run(r#"
pub struct Token;
#[contractimpl]
impl Token {
    pub fn mint(env: Env, to: Address, amount: i128) {
        to.require_auth();
        env.storage().instance().set(&KEY, &amount);
    }
}
"#);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].check_name, CHECK_NAME);
        assert_eq!(hits[0].severity, Severity::High);
        assert_eq!(hits[0].function_name, "mint");
    }

    #[test]
    fn flags_mint_with_recipient_require_auth() {
        let hits = run(r#"
pub struct Token;
#[contractimpl]
impl Token {
    pub fn mint(env: Env, recipient: Address, amount: i128) {
        recipient.require_auth();
        env.storage().instance().set(&KEY, &amount);
    }
}
"#);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].severity, Severity::High);
    }

    //  Safe cases 

    #[test]
    fn passes_when_admin_loaded_from_storage_and_authed() {
        // Admin is read from storage, then require_auth is called on it.
        let hits = run(r#"
pub struct Token;
#[contractimpl]
impl Token {
    pub fn mint(env: Env, to: Address, amount: i128) {
        let admin: Address = env.storage().instance().get(&ADMIN_KEY).unwrap();
        admin.require_auth();
        env.storage().instance().set(&KEY, &amount);
    }
}
"#);
        assert!(hits.is_empty());
    }

    #[test]
    fn passes_when_env_require_auth_used() {
        // env.require_auth() is a valid guard (not a recipient param).
        let hits = run(r#"
pub struct Token;
#[contractimpl]
impl Token {
    pub fn mint(env: Env, to: Address, amount: i128) {
        env.require_auth();
        env.storage().instance().set(&KEY, &amount);
    }
}
"#);
        // env is not a RECIPIENT_PARAM, so no flag.
        assert!(hits.is_empty());
    }

    #[test]
    fn passes_when_minter_param_used_not_recipient() {
        // A param named `minter` (not `to`/`recipient`) calling require_auth is fine.
        let hits = run(r#"
pub struct Token;
#[contractimpl]
impl Token {
    pub fn mint(env: Env, minter: Address, to: Address, amount: i128) {
        minter.require_auth();
        env.storage().instance().set(&KEY, &amount);
    }
}
"#);
        assert!(hits.is_empty());
    }

    #[test]
    fn ignores_non_mint_functions() {
        let hits = run(r#"
pub struct Token;
#[contractimpl]
impl Token {
    pub fn transfer(env: Env, to: Address, amount: i128) {
        to.require_auth();
    }
}
"#);
        assert!(hits.is_empty());
    }

    #[test]
    fn ignores_non_contractimpl_impl() {
        let hits = run(r#"
pub struct Token;
impl Token {
    pub fn mint(env: Env, to: Address, amount: i128) {
        to.require_auth();
    }
}
"#);
        assert!(hits.is_empty());
    }

    #[test]
    fn ignores_private_mint() {
        let hits = run(r#"
pub struct Token;
#[contractimpl]
impl Token {
    fn mint(env: Env, to: Address, amount: i128) {
        to.require_auth();
    }
}
"#);
        assert!(hits.is_empty());
    }
}
