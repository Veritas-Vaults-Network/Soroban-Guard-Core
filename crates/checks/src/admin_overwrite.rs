//! Admin overwrite without reading current admin first.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ExprMethodCall, File, Visibility};

const CHECK_NAME: &str = "admin-overwrite-without-read";

const ADMIN_FUNCTIONS: &[&str] = &["set_admin", "set_owner", "transfer_ownership"];

pub struct AdminOverwriteCheck;

impl Check for AdminOverwriteCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        for method in contractimpl_functions(file) {
            if !matches!(method.vis, Visibility::Public(_)) {
                continue;
            }
            let name = method.sig.ident.to_string();
            if !ADMIN_FUNCTIONS.contains(&name.as_str()) {
                continue;
            }
            let mut scan = AdminWriteScan::default();
            scan.visit_block(&method.block);
            if !scan.has_storage_write || scan.has_prior_read {
                continue;
            }
            let line = method.sig.fn_token.span().start().line;
            out.push(Finding {
                check_name: CHECK_NAME.to_string(),
                severity: Severity::High,
                file_path: String::new(),
                line,
                function_name: name.clone(),
                description: format!(
                    "Method `{name}` writes to storage without first reading the current \
                     admin. This may allow unauthorized admin overwrites."
                ),
            });
        }
        out
    }
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

fn is_storage_read_call(m: &ExprMethodCall) -> bool {
    let name = m.method.to_string();
    if !matches!(name.as_str(), "get" | "has") {
        return false;
    }
    receiver_chain_contains_storage(&m.receiver)
}

fn is_storage_write_call(m: &ExprMethodCall) -> bool {
    let name = m.method.to_string();
    if !matches!(name.as_str(), "set" | "remove") {
        return false;
    }
    receiver_chain_contains_storage(&m.receiver)
}

#[derive(Default)]
struct AdminWriteScan {
    has_prior_read: bool,
    has_storage_write: bool,
}

impl<'ast> Visit<'ast> for AdminWriteScan {
    fn visit_expr_method_call(&mut self, i: &'ast ExprMethodCall) {
        if !self.has_storage_write {
            if is_storage_read_call(i) {
                self.has_prior_read = true;
            } else if is_storage_write_call(i) {
                self.has_storage_write = true;
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

    #[test]
    fn flags_set_admin_without_read() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Address, Env};

pub struct C;

#[contractimpl]
impl C {
    pub fn set_admin(env: Env, new_admin: Address) {
        env.storage().instance().set(&"admin", &new_admin);
    }
}
"#,
        )?;
        let hits = AdminOverwriteCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].severity, Severity::High);
        Ok(())
    }

    #[test]
    fn passes_when_read_before_write() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Address, Env};

pub struct C;

#[contractimpl]
impl C {
    pub fn set_admin(env: Env, new_admin: Address) {
        let _current = env.storage().instance().get(&"admin");
        env.storage().instance().set(&"admin", &new_admin);
    }
}
"#,
        )?;
        let hits = AdminOverwriteCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }

    #[test]
    fn passes_when_has_before_write() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Address, Env};

pub struct C;

#[contractimpl]
impl C {
    pub fn set_owner(env: Env, new_owner: Address) {
        if env.storage().instance().has(&"owner") {
            env.storage().instance().set(&"owner", &new_owner);
        }
    }
}
"#,
        )?;
        let hits = AdminOverwriteCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }

    #[test]
    fn ignores_private_set_admin() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Address, Env};

pub struct C;

#[contractimpl]
impl C {
    fn set_admin(env: Env, new_admin: Address) {
        env.storage().instance().set(&"admin", &new_admin);
    }
}
"#,
        )?;
        let hits = AdminOverwriteCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }
}
