//! `extend_ttl` called inside a loop (unbounded compute cost).
//!
//! Calling `env.storage()...extend_ttl(...)` inside a loop that iterates over
//! user-supplied data makes the number of host calls unbounded, potentially
//! exhausting the transaction's compute budget.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ExprForLoop, ExprLoop, ExprMethodCall, ExprWhile, File};

const CHECK_NAME: &str = "extend-ttl-in-loop";

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

fn is_extend_ttl(m: &ExprMethodCall) -> bool {
    matches!(m.method.to_string().as_str(), "extend_ttl" | "bump")
        && receiver_chain_contains_storage(&m.receiver)
}

struct LoopVisitor<'a> {
    fn_name: String,
    loop_depth: usize,
    out: &'a mut Vec<Finding>,
}

impl<'ast> Visit<'ast> for LoopVisitor<'ast> {
    fn visit_expr_for_loop(&mut self, i: &'ast ExprForLoop) {
        self.loop_depth += 1;
        visit::visit_expr_for_loop(self, i);
        self.loop_depth -= 1;
    }

    fn visit_expr_while(&mut self, i: &'ast ExprWhile) {
        self.loop_depth += 1;
        visit::visit_expr_while(self, i);
        self.loop_depth -= 1;
    }

    fn visit_expr_loop(&mut self, i: &'ast ExprLoop) {
        self.loop_depth += 1;
        visit::visit_expr_loop(self, i);
        self.loop_depth -= 1;
    }

    fn visit_expr_method_call(&mut self, i: &'ast ExprMethodCall) {
        if self.loop_depth > 0 && is_extend_ttl(i) {
            self.out.push(Finding {
                check_name: CHECK_NAME.to_string(),
                severity: Severity::Medium,
                file_path: String::new(),
                line: i.span().start().line,
                function_name: self.fn_name.clone(),
                description: format!(
                    "`extend_ttl` is called inside a loop in `{}`. The number of host calls \
                     scales with the iteration count; if the loop iterates over user-supplied \
                     data, this can exhaust the transaction compute budget. Batch TTL \
                     extensions or move them outside the loop.",
                    self.fn_name
                ),
            });
        }
        visit::visit_expr_method_call(self, i);
    }
}

pub struct ExtendTtlInLoopCheck;

impl Check for ExtendTtlInLoopCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        for method in contractimpl_functions(file) {
            let fn_name = method.sig.ident.to_string();
            let mut v = LoopVisitor {
                fn_name,
                loop_depth: 0,
                out: &mut out,
            };
            v.visit_block(&method.block);
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
    fn flags_extend_ttl_in_for_loop() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, Address, Env, Vec};
pub struct C;
#[contractimpl]
impl C {
    pub fn refresh_all(env: Env, keys: Vec<Address>) {
        for key in keys {
            env.storage().persistent().extend_ttl(&key, 100, 200);
        }
    }
}
"#;
        let file = parse_file(src)?;
        let hits = ExtendTtlInLoopCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].severity, Severity::Medium);
        Ok(())
    }

    #[test]
    fn flags_extend_ttl_in_while_loop() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, symbol_short, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn refresh(env: Env, n: u32) {
        let mut i = 0u32;
        while i < n {
            env.storage().persistent().extend_ttl(&symbol_short!("k"), 100, 200);
            i += 1;
        }
    }
}
"#;
        let file = parse_file(src)?;
        let hits = ExtendTtlInLoopCheck.run(&file, "");
        assert_eq!(hits.len(), 1);
        Ok(())
    }

    #[test]
    fn no_finding_outside_loop() -> Result<(), syn::Error> {
        let src = r#"
use soroban_sdk::{contractimpl, symbol_short, Env};
pub struct C;
#[contractimpl]
impl C {
    pub fn refresh(env: Env) {
        env.storage().persistent().extend_ttl(&symbol_short!("k"), 100, 200);
    }
}
"#;
        let file = parse_file(src)?;
        let hits = ExtendTtlInLoopCheck.run(&file, "");
        assert!(hits.is_empty());
        Ok(())
    }
}
