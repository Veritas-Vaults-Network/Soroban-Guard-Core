//! Detects events publishing full storage values instead of meaningful deltas.

use crate::util::{binding_ident, contractimpl_functions};
use crate::{Check, Finding, Severity};
use std::collections::HashSet;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ExprMethodCall, File};

const CHECK_NAME: &str = "event-full-state";

/// Flags `events().publish()` where data is a direct storage `get` result.
pub struct EventFullStateCheck;

impl Check for EventFullStateCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        for method in contractimpl_functions(file) {
            let fn_name = method.sig.ident.to_string();
            let mut visitor = EventPublishVisitor {
                fn_name: fn_name.clone(),
                storage_vars: HashSet::new(),
                out: &mut out,
            };
            visitor.visit_block(&method.block);
        }
        out
    }
}

struct EventPublishVisitor<'a> {
    fn_name: String,
    storage_vars: HashSet<String>,
    out: &'a mut Vec<Finding>,
}

impl EventPublishVisitor<'_> {
    /// True when `expr` is a raw storage value: a `get()` result, or a local bound
    /// directly from one.
    fn is_raw_storage_value(&self, expr: &Expr) -> bool {
        if is_storage_get_result(expr) {
            return true;
        }
        match expr {
            Expr::Path(p) => p
                .path
                .get_ident()
                .is_some_and(|id| self.storage_vars.contains(&id.to_string())),
            Expr::Reference(r) => self.is_raw_storage_value(&r.expr),
            _ => false,
        }
    }
}

impl<'ast> Visit<'ast> for EventPublishVisitor<'_> {
    fn visit_local(&mut self, i: &'ast syn::Local) {
        if let Some(init) = &i.init {
            if is_storage_get_result(&init.expr) {
                if let Some(name) = binding_ident(&i.pat) {
                    self.storage_vars.insert(name);
                }
            }
        }
        visit::visit_local(self, i);
    }

    fn visit_expr_method_call(&mut self, i: &'ast ExprMethodCall) {
        if is_events_publish(i) {
            // args[0] is the topics tuple; args[1] carries the event payload.
            if let Some(data_arg) = i.args.get(1) {
                let leaks = match data_arg {
                    Expr::Tuple(t) => t.elems.iter().any(|e| self.is_raw_storage_value(e)),
                    other => self.is_raw_storage_value(other),
                };
                if leaks {
                    self.out.push(Finding {
                        check_name: CHECK_NAME.to_string(),
                        severity: Severity::Low,
                        file_path: String::new(),
                        line: i.span().start().line,
                        function_name: self.fn_name.clone(),
                        description: format!(
                            "Event data in `{}` contains a full storage value from `get()`. \
                             Publish only meaningful deltas to reduce data leakage and storage costs.",
                            self.fn_name
                        ),
                    });
                }
            }
        }
        visit::visit_expr_method_call(self, i);
    }
}

fn is_events_publish(m: &ExprMethodCall) -> bool {
    if m.method != "publish" {
        return false;
    }
    receiver_chain_contains_events(&m.receiver)
}

fn receiver_chain_contains_events(expr: &Expr) -> bool {
    match expr {
        Expr::MethodCall(m) => {
            if m.method == "events" {
                return true;
            }
            receiver_chain_contains_events(&m.receiver)
        }
        Expr::Field(f) => receiver_chain_contains_events(&f.base),
        _ => false,
    }
}

/// True for `env.storage()...get(..)`, including the usual unwrapping tail
/// (`.unwrap()`, `.unwrap_or(..)`, `.expect(..)`, `.clone()`).
fn is_storage_get_result(expr: &Expr) -> bool {
    match expr {
        Expr::MethodCall(m) => {
            if m.method == "get" {
                return receiver_chain_contains_storage(&m.receiver);
            }
            if matches!(
                m.method.to_string().as_str(),
                "unwrap"
                    | "unwrap_or"
                    | "unwrap_or_default"
                    | "unwrap_or_else"
                    | "expect"
                    | "clone"
            ) {
                return is_storage_get_result(&m.receiver);
            }
            false
        }
        _ => false,
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

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_file;

    #[test]
    fn flags_event_with_full_storage_value() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env};

pub struct C;

#[contractimpl]
impl C {
    pub fn process(env: Env) {
        let val = env.storage().persistent().get(&"key").unwrap_or(0);
        env.events().publish(("state",), (val,));
    }
}
"#,
        )?;
        let hits = EventFullStateCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].severity, Severity::Low);
        Ok(())
    }

    #[test]
    fn passes_event_with_computed_delta() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env};

pub struct C;

#[contractimpl]
impl C {
    pub fn process(env: Env) {
        let old_val = env.storage().persistent().get(&"key").unwrap_or(0);
        let new_val = old_val + 10;
        env.storage().persistent().set(&"key", &new_val);
        env.events().publish(("delta",), (10,));
    }
}
"#,
        )?;
        let hits = EventFullStateCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }

    #[test]
    fn passes_event_with_literal_data() -> Result<(), syn::Error> {
        let file = parse_file(
            r#"
use soroban_sdk::{contractimpl, Env};

pub struct C;

#[contractimpl]
impl C {
    pub fn process(env: Env) {
        env.events().publish(("event",), (42,));
    }
}
"#,
        )?;
        let hits = EventFullStateCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }
}
