//! Debug/test/dev entrypoints left in production contracts.
//!
//! Public functions whose names contain substrings like `debug`, `test`, `dev_`,
//! `backdoor`, `mock`, or `dummy` are world-callable admin backdoors.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::{File, Visibility};

const CHECK_NAME: &str = "debug-entrypoint";

const SUSPICIOUS_SUBSTRINGS: &[&str] = &["debug", "test", "dev_", "backdoor", "mock", "dummy"];

fn name_is_suspicious(name: &str) -> bool {
    let lower = name.to_lowercase();
    SUSPICIOUS_SUBSTRINGS.iter().any(|s| lower.contains(s))
}

pub struct DebugEntrypointCheck;

impl Check for DebugEntrypointCheck {
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
            if !name_is_suspicious(&name) {
                continue;
            }
            out.push(Finding {
                check_name: CHECK_NAME.to_string(),
                severity: Severity::High,
                file_path: String::new(),
                line: method.sig.fn_token.span().start().line,
                function_name: name.clone(),
                description: format!(
                    "Public method `{name}` has a debug/test/dev name and should not exist \
                     in a production contract. It is world-callable and may act as an \
                     admin backdoor. Remove it or make it private before deployment."
                ),
            });
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Check;
    use syn::parse_file;

    #[test]
    fn flags_debug_fn() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn debug_state(env: Env) {}
}
"#;
        let file = parse_file(src)?;
        let hits = DebugEntrypointCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].severity, Severity::High);
        Ok(())
    }

    #[test]
    fn flags_dev_mint() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn dev_mint(env: Env) {}
}
"#;
        let file = parse_file(src)?;
        let hits = DebugEntrypointCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        Ok(())
    }

    #[test]
    fn flags_backdoor() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn backdoor(env: Env) {}
}
"#;
        let file = parse_file(src)?;
        let hits = DebugEntrypointCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        Ok(())
    }

    #[test]
    fn no_finding_for_normal_fn() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn transfer(env: Env) {}
}
"#;
        let file = parse_file(src)?;
        let hits = DebugEntrypointCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }

    #[test]
    fn no_finding_for_private_debug_fn() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, Env};
pub struct C;
#[contractimpl]
impl C {
    fn debug_helper(env: Env) {}
}
"#;
        let file = parse_file(src)?;
        let hits = DebugEntrypointCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }
}
