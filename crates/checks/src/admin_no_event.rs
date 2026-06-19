//! Detects admin-transfer functions that do not emit an event.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{ExprMethodCall, File};

const CHECK_NAME: &str = "admin-no-event";

const ADMIN_FN_NAMES: &[&str] = &["set_admin", "transfer_ownership", "change_admin"];

/// Flags `pub fn set_admin`, `pub fn transfer_ownership`, or `pub fn change_admin`
/// in `#[contractimpl]` blocks that do not call `events().publish(...)`.
pub struct AdminNoEventCheck;

impl Check for AdminNoEventCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        for method in contractimpl_functions(file) {
            let fn_name = method.sig.ident.to_string();
            if !ADMIN_FN_NAMES.contains(&fn_name.as_str()) {
                continue;
            }
            // Check whether the function body calls events().publish(...)
            let mut checker = EventPublishChecker { found: false };
            checker.visit_block(&method.block);
            if !checker.found {
                out.push(Finding {
                    check_name: CHECK_NAME.to_string(),
                    severity: Severity::Low,
                    file_path: String::new(),
                    line: method.sig.span().start().line,
                    function_name: fn_name.clone(),
                    description: format!(
                        "`{}` changes the contract admin/owner without emitting an event. \
                         Add `env.events().publish(...)` to make the change visible to \
                         off-chain indexers and users.",
                        fn_name
                    ),
                });
            }
        }
        out
    }
}

struct EventPublishChecker {
    found: bool,
}

impl Visit<'_> for EventPublishChecker {
    fn visit_expr_method_call(&mut self, i: &ExprMethodCall) {
        if i.method == "publish" && receiver_contains_events(&i.receiver) {
            self.found = true;
        }
        syn::visit::visit_expr_method_call(self, i);
    }
}

fn receiver_contains_events(expr: &syn::Expr) -> bool {
    match expr {
        syn::Expr::MethodCall(m) => {
            if m.method == "events" {
                return true;
            }
            receiver_contains_events(&m.receiver)
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
        let file = parse_file(src).unwrap();
        AdminNoEventCheck.run(&file, src)
    }

    #[test]
    fn flags_set_admin_without_event() {
        let hits = run(r#"
use soroban_sdk::{contractimpl, Address, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn set_admin(env: Env, new_admin: Address) {
        env.storage().instance().set(&"admin", &new_admin);
    }
}
"#);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].check_name, CHECK_NAME);
        assert_eq!(hits[0].severity, Severity::Low);
        assert_eq!(hits[0].function_name, "set_admin");
    }

    #[test]
    fn flags_transfer_ownership_without_event() {
        let hits = run(r#"
use soroban_sdk::{contractimpl, Address, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn transfer_ownership(env: Env, new_owner: Address) {
        env.storage().instance().set(&"owner", &new_owner);
    }
}
"#);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].function_name, "transfer_ownership");
    }

    #[test]
    fn does_not_flag_set_admin_with_event() {
        let hits = run(r#"
use soroban_sdk::{contractimpl, Address, Env, symbol_short};
pub struct C;
#[contractimpl]
impl C {
    pub fn set_admin(env: Env, new_admin: Address) {
        env.storage().instance().set(&"admin", &new_admin);
        env.events().publish((symbol_short!("admin"),), new_admin);
    }
}
"#);
        assert!(hits.is_empty());
    }

    #[test]
    fn does_not_flag_unrelated_functions() {
        let hits = run(r#"
use soroban_sdk::{contractimpl, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn deposit(env: Env, amount: i128) {
        env.storage().instance().set(&"balance", &amount);
    }
}
"#);
        assert!(hits.is_empty());
    }
}
