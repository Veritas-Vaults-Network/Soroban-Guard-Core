//! Admin address compared with == instead of require_auth (bypasses signature verification).

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Block, Expr, ExprBinary, File, Pat, PatType};

const CHECK_NAME: &str = "address-cmp-instead-of-auth";

pub struct AddressCmpInsteadOfAuthCheck;

impl Check for AddressCmpInsteadOfAuthCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        for method in contractimpl_functions(file) {
            let fn_name = method.sig.ident.to_string();
            let param_names = extract_address_params(&method.sig.inputs);
            let has_auth = has_require_auth_call(&method.block);
            let mut v = AddressCmpVisitor {
                fn_name: fn_name.clone(),
                param_names,
                has_auth,
                out: &mut out,
            };
            v.visit_block(&method.block);
        }
        out
    }
}

fn extract_address_params(inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>) -> Vec<String> {
    let mut names = Vec::new();
    for arg in inputs {
        if let syn::FnArg::Typed(PatType { pat, ty, .. }) = arg {
            if let Pat::Ident(ident) = &**pat {
                if is_address_type(ty) {
                    names.push(ident.ident.to_string());
                }
            }
        }
    }
    names
}

fn is_address_type(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(p) => {
            p.path
                .segments
                .last()
                .is_some_and(|s| s.ident == "Address")
        }
        _ => false,
    }
}

fn has_require_auth_call(block: &Block) -> bool {
    let mut v = AuthCallFinder { found: false };
    v.visit_block(block);
    v.found
}

struct AuthCallFinder {
    found: bool,
}

impl Visit<'_> for AuthCallFinder {
    fn visit_expr_method_call(&mut self, i: &syn::ExprMethodCall) {
        if i.method == "require_auth" {
            self.found = true;
        }
        visit::visit_expr_method_call(self, i);
    }
}

struct AddressCmpVisitor<'a> {
    fn_name: String,
    param_names: Vec<String>,
    has_auth: bool,
    out: &'a mut Vec<Finding>,
}

impl Visit<'_> for AddressCmpVisitor<'_> {
    fn visit_expr_binary(&mut self, i: &ExprBinary) {
        if matches!(i.op, syn::BinOp::Eq(_)) && !self.has_auth {
            let left_is_param = is_address_param(&i.left, &self.param_names);
            let right_is_storage = is_storage_read(&i.right);

            if left_is_param && right_is_storage {
                self.out.push(Finding {
                    check_name: CHECK_NAME.to_string(),
                    severity: Severity::High,
                    file_path: String::new(),
                    line: i.span().start().line,
                    function_name: self.fn_name.clone(),
                    description: format!(
                        "Method `{}` compares an Address parameter with a storage-read value \
                         using ==, bypassing Soroban's host-level signature verification. \
                         Use `require_auth()` instead.",
                        self.fn_name
                    ),
                });
            }
        }
        visit::visit_expr_binary(self, i);
    }
}

fn is_address_param(expr: &Expr, param_names: &[String]) -> bool {
    if let Expr::Path(p) = expr {
        if let Some(ident) = p.path.segments.last() {
            return param_names.contains(&ident.ident.to_string());
        }
    }
    false
}

fn is_storage_read(expr: &Expr) -> bool {
    if let Expr::MethodCall(m) = expr {
        if m.method == "get" {
            return receiver_chain_contains_storage(&m.receiver);
        }
    }
    false
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Check;
    use syn::parse_file;

    #[test]
    fn flags_address_comparison_without_auth() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Address, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn bad(env: Env, caller: Address) {
        let admin: Address = env.storage().persistent().get(&"admin").unwrap();
        if caller == admin {
            let _ = ();
        }
    }
}
"#,
        )?;
        let hits = AddressCmpInsteadOfAuthCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].severity, Severity::High);
        Ok(())
    }

    #[test]
    fn no_finding_with_require_auth() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Address, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn good(env: Env, caller: Address) {
        env.require_auth();
        let admin: Address = env.storage().persistent().get(&"admin").unwrap();
        let _ = (caller, admin);
    }
}
"#,
        )?;
        let hits = AddressCmpInsteadOfAuthCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }

    #[test]
    fn no_finding_for_non_address_comparison() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn good(env: Env, amount: i128) {
        if amount == 100 {
            let _ = ();
        }
    }
}
"#,
        )?;
        let hits = AddressCmpInsteadOfAuthCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }
}
