//! Detects time-lock parameters typed as `u32` instead of `u64`.
//!
//! `env.ledger().timestamp()` returns a `u64`. Storing or accepting a timestamp
//! as `u32` silently truncates values after year 2106, causing time-locks to
//! either never expire or expire immediately.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::{File, FnArg, Pat, PathArguments, Type};

const CHECK_NAME: &str = "timestamp-truncation";

const TIME_LOCK_NAMES: &[&str] = &[
    "unlock_time",
    "lock_until",
    "expiry",
    "deadline",
    "valid_until",
];

pub struct TimestampTruncationCheck;

impl Check for TimestampTruncationCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        for method in contractimpl_functions(file) {
            let fn_name = method.sig.ident.to_string();
            for arg in &method.sig.inputs {
                let FnArg::Typed(pat_type) = arg else {
                    continue;
                };
                let Pat::Ident(pat_ident) = pat_type.pat.as_ref() else {
                    continue;
                };
                let param_name = pat_ident.ident.to_string();
                if !TIME_LOCK_NAMES.contains(&param_name.as_str()) {
                    continue;
                }
                if is_u32(&pat_type.ty) {
                    let line = pat_type.ty.span().start().line;
                    out.push(Finding {
                        check_name: CHECK_NAME.to_string(),
                        severity: Severity::Medium,
                        file_path: String::new(),
                        line,
                        function_name: fn_name.clone(),
                        description: format!(
                            "Parameter `{param_name}` in `{fn_name}` is typed as `u32`. \
                             `env.ledger().timestamp()` returns `u64`; using `u32` silently \
                             truncates timestamps after year 2106, breaking time-lock logic. \
                             Use `u64` instead."
                        ),
                    });
                }
            }
        }
        out
    }
}

fn is_u32(ty: &Type) -> bool {
    let Type::Path(type_path) = ty else {
        return false;
    };
    let Some(seg) = type_path.path.segments.last() else {
        return false;
    };
    seg.ident == "u32" && matches!(seg.arguments, PathArguments::None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Check;
    use syn::parse_file;

    #[test]
    fn flags_unlock_time_u32() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
pub struct C;
#[contractimpl]
impl C {
    pub fn lock(env: Env, unlock_time: u32) {}
}
"#,
        )?;
        let hits = TimestampTruncationCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].severity, Severity::Medium);
        assert_eq!(hits[0].function_name, "lock");
        Ok(())
    }

    #[test]
    fn flags_all_time_lock_names() -> Result<(), syn::Error> {
        let src = r#"
pub struct C;
#[contractimpl]
impl C {
    pub fn a(env: Env, lock_until: u32) {}
    pub fn b(env: Env, expiry: u32) {}
    pub fn c(env: Env, deadline: u32) {}
    pub fn d(env: Env, valid_until: u32) {}
}
"#;
        let file = parse_file(src)?;
        let hits = TimestampTruncationCheck.run(&file, "");
        assert_eq!(hits.len(), 4);
        Ok(())
    }

    #[test]
    fn passes_when_u64() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
pub struct C;
#[contractimpl]
impl C {
    pub fn lock(env: Env, unlock_time: u64) {}
}
"#,
        )?;
        let hits = TimestampTruncationCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }

    #[test]
    fn ignores_unrelated_u32_param() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
pub struct C;
#[contractimpl]
impl C {
    pub fn deposit(env: Env, amount: u32) {}
}
"#,
        )?;
        let hits = TimestampTruncationCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }
}
