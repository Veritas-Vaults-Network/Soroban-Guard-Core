//! Admin/owner data stored in temporary storage — expires with TTL, leaving contract without admin.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ExprMethodCall, File};

const CHECK_NAME: &str = "admin-in-temp";

pub struct AdminInTempCheck;

impl Check for AdminInTempCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        for method in contractimpl_functions(file) {
            let mut v = AdminInTempVisitor {
                fn_name: method.sig.ident.to_string(),
                out: &mut out,
            };
            v.visit_block(&method.block);
        }
        out
    }
}

fn receiver_chain_contains_temporary(expr: &Expr) -> bool {
    match expr {
        Expr::MethodCall(m) => {
            m.method == "temporary" || receiver_chain_contains_temporary(&m.receiver)
        }
        Expr::Field(f) => receiver_chain_contains_temporary(&f.base),
        _ => false,
    }
}

/// Returns the string representation of the first argument to `.set(key, val)`.
fn first_arg_str(m: &ExprMethodCall) -> Option<String> {
    let arg = m.args.first()?;
    Some(match arg {
        Expr::Reference(r) => expr_to_string(&r.expr),
        other => expr_to_string(other),
    })
}

fn expr_to_string(expr: &Expr) -> String {
    match expr {
        Expr::Path(p) => p
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default(),
        Expr::Lit(l) => match &l.lit {
            syn::Lit::Str(s) => s.value(),
            _ => String::new(),
        },
        Expr::Macro(m) => m
            .mac
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default(),
        _ => String::new(),
    }
}

fn key_looks_like_admin(key: &str) -> bool {
    let lower = key.to_lowercase();
    lower.contains("admin") || lower.contains("owner") || lower.contains("role")
}

struct AdminInTempVisitor<'a> {
    fn_name: String,
    out: &'a mut Vec<Finding>,
}

impl Visit<'_> for AdminInTempVisitor<'_> {
    fn visit_expr_method_call(&mut self, i: &ExprMethodCall) {
        if i.method == "set" && receiver_chain_contains_temporary(&i.receiver) {
            if let Some(key) = first_arg_str(i) {
                if key_looks_like_admin(&key) {
                    self.out.push(Finding {
                        check_name: CHECK_NAME.to_string(),
                        severity: Severity::High,
                        file_path: String::new(),
                        line: i.span().start().line,
                        function_name: self.fn_name.clone(),
                        description: format!(
                            "Method `{}` stores an admin/owner/role key (`{}`) in \
                             `env.storage().temporary()`. Temporary storage expires with TTL, \
                             potentially leaving the contract without an admin. Use \
                             `persistent()` or `instance()` instead.",
                            self.fn_name, key
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

    #[test]
    fn flags_admin_key_in_temporary() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, symbol_short, Address, Env};
pub struct C;
const ADMIN: soroban_sdk::Symbol = symbol_short!("admin");
#[contractimpl]
impl C {
    pub fn set_admin(env: Env, new_admin: Address) {
        env.storage().temporary().set(&ADMIN, &new_admin);
    }
}
"#,
        )?;
        let hits = AdminInTempCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].severity, Severity::High);
        assert!(hits[0].description.contains("ADMIN"));
        Ok(())
    }

    #[test]
    fn flags_owner_string_key_in_temporary() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Address, Env, Symbol};
pub struct C;
#[contractimpl]
impl C {
    pub fn init(env: Env, owner: Address) {
        let key = Symbol::new(&env, "owner");
        env.storage().temporary().set(&key, &owner);
    }
}
"#,
        )?;
        let hits = AdminInTempCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        Ok(())
    }

    #[test]
    fn no_finding_for_persistent_admin() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, symbol_short, Address, Env};
pub struct C;
const ADMIN: soroban_sdk::Symbol = symbol_short!("admin");
#[contractimpl]
impl C {
    pub fn set_admin(env: Env, new_admin: Address) {
        env.require_auth();
        env.storage().persistent().set(&ADMIN, &new_admin);
    }
}
"#,
        )?;
        let hits = AdminInTempCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }

    #[test]
    fn no_finding_for_non_admin_temp_key() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, symbol_short, Env};
pub struct C;
const COUNTER: soroban_sdk::Symbol = symbol_short!("cnt");
#[contractimpl]
impl C {
    pub fn tick(env: Env) {
        env.storage().temporary().set(&COUNTER, &1u32);
    }
}
"#,
        )?;
        let hits = AdminInTempCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }
}
