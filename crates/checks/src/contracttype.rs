//! Detects structs and enums stored in Soroban persistent/instance storage without #[contracttype].
//!
//! Custom types stored in Soroban storage must derive #[contracttype] to be serialized
//! correctly by the Soroban host. Storing a type without this attribute will cause runtime errors.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{
    Attribute, Expr, ExprMethodCall, File, Item, ItemEnum, ItemStruct, Pat, ReturnType, Stmt,
};

const CHECK_NAME: &str = "missing-contracttype";

/// Flags struct and enum definitions that appear to be used as storage values
/// (passed to `env.storage()...set(...)`) but do not have `#[contracttype]` or `#[derive(...)]`
/// containing contracttype.
pub struct MissingContracttypeCheck;

impl Check for MissingContracttypeCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();

        // Collect all types used in storage.set() calls
        let mut storage_types = std::collections::HashSet::new();
        for method in contractimpl_functions(file) {
            let mut scanner = StorageTypeScanner::default();
            scanner.visit_block(&method.block);
            storage_types.extend(scanner.types);
        }

        // Check each struct and enum for #[contracttype] attribute
        for item in &file.items {
            match item {
                Item::Struct(s) => {
                    let name = s.ident.to_string();
                    if storage_types.contains(&name) && !has_contracttype_attr(s) {
                        out.push(Finding {
                            check_name: CHECK_NAME.to_string(),
                            severity: Severity::Medium,
                            file_path: String::new(),
                            line: s.span().start().line,
                            function_name: String::new(),
                            description: format!(
                                "Struct `{}` is stored in env.storage() but lacks #[contracttype] \
                                 attribute. This will cause runtime serialization errors.",
                                name
                            ),
                        });
                    }
                }
                Item::Enum(e) => {
                    let name = e.ident.to_string();
                    if storage_types.contains(&name) && !has_contracttype_attr(e) {
                        out.push(Finding {
                            check_name: CHECK_NAME.to_string(),
                            severity: Severity::Medium,
                            file_path: String::new(),
                            line: e.span().start().line,
                            function_name: String::new(),
                            description: format!(
                                "Enum `{}` is stored in env.storage() but lacks #[contracttype] \
                                 attribute. This will cause runtime serialization errors.",
                                name
                            ),
                        });
                    }
                }
                _ => {}
            }
        }

        out
    }
}

/// Check if a struct has #[contracttype] or #[derive(...contracttype...)]
fn has_contracttype_attr<T: HasAttrs>(item: &T) -> bool {
    for attr in item.attrs() {
        // Direct #[contracttype]
        if attr.path().is_ident("contracttype") {
            return true;
        }
        // #[derive(...contracttype...)]
        if attr.path().is_ident("derive") {
            if let syn::Meta::List(meta_list) = &attr.meta {
                if meta_list.tokens.to_string().contains("contracttype") {
                    return true;
                }
            }
        }
    }
    false
}

trait HasAttrs {
    fn attrs(&self) -> &[Attribute];
}

impl HasAttrs for ItemStruct {
    fn attrs(&self) -> &[Attribute] {
        &self.attrs
    }
}

impl HasAttrs for ItemEnum {
    fn attrs(&self) -> &[Attribute] {
        &self.attrs
    }
}

#[derive(Default)]
struct StorageTypeScanner {
    types: Vec<String>,
}

impl<'ast> Visit<'ast> for StorageTypeScanner {
    fn visit_expr_method_call(&mut self, i: &'ast ExprMethodCall) {
        // Check for .set(...) calls on storage objects
        if i.method == "set" && is_receiver_from_storage(&i.receiver) {
            // The second argument is typically the value being stored
            if i.args.len() >= 2 {
                if let Some(type_name) = extract_type_name_from_expr(&i.args[1]) {
                    self.types.push(type_name);
                }
            }
        }
        visit::visit_expr_method_call(self, i);
    }
}

fn is_receiver_from_storage(expr: &Expr) -> bool {
    match expr {
        Expr::MethodCall(m) => {
            if m.method == "storage" || m.method == "instance" || m.method == "persistent" {
                return true;
            }
            is_receiver_from_storage(&m.receiver)
        }
        Expr::Field(f) => is_receiver_from_storage(&f.base),
        _ => false,
    }
}

/// Try to extract a type name from an expression
fn extract_type_name_from_expr(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Path(p) => {
            if p.path.segments.len() == 1 {
                Some(p.path.segments[0].ident.to_string())
            } else {
                None
            }
        }
        Expr::Struct(es) => {
            if let syn::Path { segments, .. } = &es.path {
                if segments.len() == 1 {
                    Some(segments[0].ident.to_string())
                } else {
                    None
                }
            } else {
                None
            }
        }
        Expr::Reference(r) => extract_type_name_from_expr(&r.expr),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_file;

    #[test]
    fn detects_missing_contracttype_on_struct() {
        let code = r#"
struct MyData {
    value: i32,
}

#[contractimpl]
impl MyContract {
    pub fn store(env: Env, data: MyData) {
        let d = MyData { value: 42 };
        env.storage().instance().set(&key, &d);
    }
}
        "#;
        let file = parse_file(code).unwrap();
        let check = MissingContracttypeCheck;
        let findings = check.run(&file, code);
        assert!(findings.len() >= 1);
        assert!(findings.iter().any(|f| f.function_name.is_empty()
            && f.description.contains("MyData")));
    }

    #[test]
    fn allows_struct_with_contracttype() {
        let code = r#"
#[contracttype]
struct MyData {
    value: i32,
}

#[contractimpl]
impl MyContract {
    pub fn store(env: Env, data: MyData) {
        let d = MyData { value: 42 };
        env.storage().instance().set(&key, &d);
    }
}
        "#;
        let file = parse_file(code).unwrap();
        let check = MissingContracttypeCheck;
        let findings = check.run(&file, code);
        assert!(findings.is_empty());
    }

    #[test]
    fn allows_struct_with_derive_contracttype() {
        let code = r#"
#[derive(Clone, contracttype)]
struct MyData {
    value: i32,
}

#[contractimpl]
impl MyContract {
    pub fn store(env: Env, data: MyData) {
        let d = MyData { value: 42 };
        env.storage().instance().set(&key, &d);
    }
}
        "#;
        let file = parse_file(code).unwrap();
        let check = MissingContracttypeCheck;
        let findings = check.run(&file, code);
        assert!(findings.is_empty());
    }
}
