//! Missing `env.require_auth()` before storage writes in `#[contractimpl]` methods.

use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Block, Expr, ExprMethodCall, File, ImplItem, Item, ItemImpl};

const CHECK_NAME: &str = "missing-require-auth";

/// Flags `#[contractimpl]` methods that write via `env.storage()` without calling
/// `env.require_auth()` on the `Env` parameter (typically named `env`).
pub struct MissingRequireAuthCheck;

impl Check for MissingRequireAuthCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        for item in &file.items {
            let Item::Impl(item_impl) = item else {
                continue;
            };
            if !is_contractimpl(item_impl) {
                continue;
            }
            for impl_item in &item_impl.items {
                let ImplItem::Fn(method) = impl_item else {
                    continue;
                };
                let mut scan = FuncBodyScan::default();
                scan.visit_block(&method.block);
                if !scan.storage_write || scan.env_require_auth {
                    continue;
                }
                let line = first_storage_write_line(&method.block).unwrap_or_else(|| {
                    method.sig.ident.span().start().line
                });
                let fn_name = method.sig.ident.to_string();
                out.push(Finding {
                    check_name: CHECK_NAME.to_string(),
                    severity: Severity::High,
                    file_path: String::new(),
                    line,
                    function_name: fn_name.clone(),
                    description: format!(
                        "Method `{fn_name}` writes to `env.storage()` but does not call \
                         `env.require_auth()`. Callers may mutate contract state without proving \
                         they are authorized."
                    ),
                });
            }
        }
        out
    }
}

fn is_contractimpl(item_impl: &ItemImpl) -> bool {
    item_impl
        .attrs
        .iter()
        .any(|attr| path_is_contractimpl(attr.path()))
}

fn path_is_contractimpl(path: &syn::Path) -> bool {
    path
        .segments
        .last()
        .is_some_and(|s| s.ident == "contractimpl")
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

fn is_storage_mutation_call(m: &ExprMethodCall) -> bool {
    let name = m.method.to_string();
    if !matches!(
        name.as_str(),
        "set" | "remove" | "extend_ttl" | "bump" | "append"
    ) {
        return false;
    }
    receiver_chain_contains_storage(&m.receiver)
}

fn is_env_require_auth(m: &ExprMethodCall) -> bool {
    if m.method != "require_auth" {
        return false;
    }
    match &*m.receiver {
        Expr::Path(p) => p.path.is_ident("env"),
        _ => false,
    }
}

#[derive(Default)]
struct FuncBodyScan {
    storage_write: bool,
    env_require_auth: bool,
}

impl<'ast> Visit<'ast> for FuncBodyScan {
    fn visit_expr_method_call(&mut self, i: &'ast ExprMethodCall) {
        if is_storage_mutation_call(i) {
            self.storage_write = true;
        }
        if is_env_require_auth(i) {
            self.env_require_auth = true;
        }
        visit::visit_expr_method_call(self, i);
    }
}

struct FirstStorageWrite {
    line: Option<usize>,
}

impl<'ast> Visit<'ast> for FirstStorageWrite {
    fn visit_expr_method_call(&mut self, i: &'ast ExprMethodCall) {
        if self.line.is_none() && is_storage_mutation_call(i) {
            self.line = Some(i.span().start().line);
        }
        visit::visit_expr_method_call(self, i);
    }
}

fn first_storage_write_line(block: &Block) -> Option<usize> {
    let mut v = FirstStorageWrite { line: None };
    v.visit_block(block);
    v.line
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Check;
    use syn::parse_file;

    fn run_on_src(src: &str) -> Vec<Finding> {
        let file = parse_file(src).unwrap();
        MissingRequireAuthCheck.run(&file, src)
    }

    #[test]
    fn flags_persistent_set_without_env_require_auth() {
        let hits = run_on_src(
            r#"
use soroban_sdk::{contractimpl, Env, Symbol};

pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn set_balance(env: Env, amount: i128) {
        env.storage().persistent().set(&Symbol::new(&env, "bal"), &amount);
    }
}
"#,
        );
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].function_name, "set_balance");
        assert_eq!(hits[0].severity, Severity::High);
        assert_eq!(hits[0].check_name, CHECK_NAME);
    }

    #[test]
    fn passes_when_env_require_auth_present() {
        let hits = run_on_src(
            r#"
use soroban_sdk::{contractimpl, Address, Env, Symbol};

pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn set_balance(env: Env, user: Address, amount: i128) {
        env.require_auth();
        env.storage().persistent().set(&Symbol::new(&env, "bal"), &amount);
    }
}
"#,
        );
        assert!(hits.is_empty());
    }

    #[test]
    fn still_flags_when_only_address_require_auth() {
        let hits = run_on_src(
            r#"
use soroban_sdk::{contractimpl, Address, Env, Symbol};

pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn set_balance(env: Env, user: Address, amount: i128) {
        user.require_auth();
        env.storage().persistent().set(&Symbol::new(&env, "bal"), &amount);
    }
}
"#,
        );
        assert_eq!(
            hits.len(),
            1,
            "`user.require_auth()` is not `env.require_auth()` per this check"
        );
    }

    #[test]
    fn still_flags_when_env_require_auth_for_args_only() {
        let hits = run_on_src(
            r#"
use soroban_sdk::{contractimpl, Address, Env, Symbol};

pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn set_balance(env: Env, user: Address, amount: i128) {
        env.require_auth_for_args((user, amount));
        env.storage().persistent().set(&Symbol::new(&env, "bal"), &amount);
    }
}
"#,
        );
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn recognizes_soroban_sdk_contractimpl_path() {
        let hits = run_on_src(
            r#"
use soroban_sdk::{contractimpl, Env, Symbol};

pub struct Contract;

#[soroban_sdk::contractimpl]
impl Contract {
    pub fn bad(env: Env) {
        env.storage().instance().set(&Symbol::new(&env, "k"), &0u32);
    }
}
"#,
        );
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].function_name, "bad");
    }

    #[test]
    fn ignores_non_contractimpl_impl() {
        let hits = run_on_src(
            r#"
use soroban_sdk::{Env, Symbol};

pub struct Contract;

impl Contract {
    pub fn helper(env: Env) {
        env.storage().persistent().set(&Symbol::new(&env, "k"), &0u32);
    }
}
"#,
        );
        assert!(hits.is_empty());
    }
}
