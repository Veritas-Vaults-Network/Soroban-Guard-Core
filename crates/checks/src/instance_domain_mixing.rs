//! Detects mixed unrelated data domains stored in `env.storage().instance()`.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use std::collections::HashSet;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ExprLit, ExprMethodCall, File, Lit};

const CHECK_NAME: &str = "instance-domain-mixing";

pub struct InstanceDomainMixingCheck;

impl Check for InstanceDomainMixingCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut groups = HashSet::new();
        let mut first_line = None;

        for method in contractimpl_functions(file) {
            let mut visitor = InstanceSetVisitor {
                groups: &mut groups,
                first_line: &mut first_line,
            };
            visitor.visit_block(&method.block);
        }

        if groups.len() > 2 {
            let line = first_line.unwrap_or(1);
            let description = format!(
                "Multiple unrelated instance-storage domains are written in `env.storage().instance()`: {}. Use a dedicated storage domain or separate contract storage for logically unrelated data.",
                groups.iter().cloned().collect::<Vec<_>>().join(", ")
            );
            return vec![Finding {
                check_name: CHECK_NAME.to_string(),
                severity: Severity::Medium,
                file_path: String::new(),
                line,
                function_name: String::new(),
                description,
            }];
        }

        Vec::new()
    }
}

struct InstanceSetVisitor<'a> {
    groups: &'a mut HashSet<&'static str>,
    first_line: &'a mut Option<usize>,
}

impl<'a> Visit<'_> for InstanceSetVisitor<'a> {
    fn visit_expr_method_call(&mut self, i: &ExprMethodCall) {
        if i.method == "set" && receiver_chain_contains_instance(&i.receiver) {
            if let Some(group) = classify_instance_key(&i.args[0]) {
                self.groups.insert(group);
                if self.first_line.is_none() {
                    *self.first_line = Some(i.span().start().line);
                }
            }
        }
        visit::visit_expr_method_call(self, i);
    }
}

fn receiver_chain_contains_instance(expr: &Expr) -> bool {
    match expr {
        Expr::MethodCall(m) => {
            if m.method == "instance" {
                return true;
            }
            receiver_chain_contains_instance(&m.receiver)
        }
        Expr::Field(f) => receiver_chain_contains_instance(&f.base),
        _ => false,
    }
}

fn classify_instance_key(expr: &Expr) -> Option<&'static str> {
    let string = match expr {
        Expr::Reference(r) => &*r.expr,
        _ => expr,
    };

    if let Expr::Lit(ExprLit {
        lit: Lit::Str(lit_str),
        ..
    }) = string
    {
        let value = lit_str.value().to_lowercase();
        if value.contains("balance") || value.contains("allowance") {
            return Some("balance/allowance");
        }
        if value.contains("vote") || value.contains("proposal") {
            return Some("vote/proposal");
        }
        if value.contains("config") || value.contains("admin") {
            return Some("config/admin");
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Check;
    use syn::parse_file;

    fn run_on_src(src: &str) -> Result<Vec<Finding>, syn::Error> {
        let file = parse_file(src)?;
        Ok(InstanceDomainMixingCheck.run(&file, src))
    }

    #[test]
    fn flags_three_unrelated_instance_domains() -> Result<(), syn::Error> {
        let hits = run_on_src(
            r#"
use soroban_sdk::{contractimpl, Env};

pub struct C;

#[contractimpl]
impl C {
    pub fn process(env: Env) {
        env.storage().instance().set(&"balance_key", &42u32);
        env.storage().instance().set(&"proposal_id", &1u32);
        env.storage().instance().set(&"config_admin", &true);
    }
}
"#,
        )?;
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].severity, Severity::Medium);
        Ok(())
    }

    #[test]
    fn does_not_flag_two_related_domains() -> Result<(), syn::Error> {
        let hits = run_on_src(
            r#"
use soroban_sdk::{contractimpl, Env};

pub struct C;

#[contractimpl]
impl C {
    pub fn process(env: Env) {
        env.storage().instance().set(&"balance_key", &42u32);
        env.storage().instance().set(&"allowance_key", &7u32);
    }
}
"#,
        )?;
        assert!(hits.is_empty());
        Ok(())
    }

    #[test]
    fn does_not_flag_single_domain() -> Result<(), syn::Error> {
        let hits = run_on_src(
            r#"
use soroban_sdk::{contractimpl, Env};

pub struct C;

#[contractimpl]
impl C {
    pub fn process(env: Env) {
        env.storage().instance().set(&"proposal_id", &1u32);
    }
}
"#,
        )?;
        assert!(hits.is_empty());
        Ok(())
    }
}
