//! Detects structs that store `Env` handles in fields.

use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::{File, Item, Type, TypePath};

const CHECK_NAME: &str = "env-in-struct";

pub struct EnvInStructCheck;

impl Check for EnvInStructCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();

        for item in &file.items {
            let Item::Struct(item_struct) = item else {
                continue;
            };

            let struct_name = item_struct.ident.to_string();
            if let Some((field_name, line)) = item_struct
                .fields
                .iter()
                .filter_map(|field| {
                    if is_env_type(&field.ty) {
                        let name = field
                            .ident
                            .as_ref()
                            .map(|id| id.to_string())
                            .unwrap_or_default();
                        Some((name, field.ty.span().start().line))
                    } else {
                        None
                    }
                })
                .next()
            {
                out.push(Finding {
                    check_name: CHECK_NAME.to_string(),
                    severity: Severity::Medium,
                    file_path: String::new(),
                    line,
                    function_name: struct_name.clone(),
                    description: format!(
                        "Struct `{}` stores an `Env` handle in field `{}`. `Env` handles are only valid for the duration of a single contract invocation and must not be persisted in a struct.",
                        struct_name, field_name
                    ),
                });
            }
        }

        out
    }
}

fn is_env_type(ty: &Type) -> bool {
    match ty {
        Type::Path(TypePath { path, .. }) => path_is_env(path),
        Type::Reference(rt) => match &*rt.elem {
            Type::Path(TypePath { path, .. }) => path_is_env(path),
            _ => false,
        },
        _ => false,
    }
}

fn path_is_env(path: &syn::Path) -> bool {
    path.segments
        .last()
        .is_some_and(|segment| segment.ident == "Env")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Check;
    use syn::parse_file;

    #[test]
    fn flags_struct_with_env_field() {
        let file = parse_file(
            r#"
use soroban_sdk::{contract, contractimpl, Env};
pub struct StoredEnv {
    env: Env,
}

#[contract]
pub struct C;
#[contractimpl]
impl C {
    pub fn test(_env: Env) {}
}
"#,
        )
        .unwrap();

        let hits = EnvInStructCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].check_name, CHECK_NAME);
        assert_eq!(hits[0].severity, Severity::Medium);
        assert_eq!(hits[0].function_name, "StoredEnv");
    }

    #[test]
    fn flags_struct_with_env_reference_field() {
        let file = parse_file(
            r#"
use soroban_sdk::{contract, contractimpl, Env};
pub struct StoredEnvRef<'a> {
    env: &'a Env,
}

#[contract]
pub struct C;
#[contractimpl]
impl C {
    pub fn test(_env: Env) {}
}
"#,
        )
        .unwrap();

        let hits = EnvInStructCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn flags_struct_with_fully_qualified_env_field() {
        let file = parse_file(
            r#"
pub struct StoredEnv {
    env: soroban_sdk::Env,
}
"#,
        )
        .unwrap();

        let hits = EnvInStructCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn ignores_struct_without_env_field() {
        let file = parse_file(
            r#"
pub struct SafeStruct {
    count: u32,
}
"#,
        )
        .unwrap();

        let hits = EnvInStructCheck.run(&file, "");
        assert!(hits.is_empty());
    }
}
