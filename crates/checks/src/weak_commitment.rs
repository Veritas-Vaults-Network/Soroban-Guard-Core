//! Detects `env.crypto().sha256()` used as commitment without a nonce.
//!
//! Using `env.crypto().sha256(data)` as a commitment without including a random
//! nonce in the preimage is vulnerable to preimage attacks. An attacker can
//! brute-force small or predictable inputs.

use crate::util::{binding_ident, contractimpl_functions};
use crate::{Check, Finding, Severity};
use std::collections::HashSet;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ExprMethodCall, File};

const CHECK_NAME: &str = "weak-commitment";

/// Flags `env.crypto().sha256(...)` calls with simple arguments (no nonce).
pub struct WeakCommitmentCheck;

impl Check for WeakCommitmentCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        for method in contractimpl_functions(file) {
            let fn_name = method.sig.ident.to_string();
            let mut scan = CommitmentScan {
                fn_name,
                compound_locals: compound_locals(&method.block),
                out: &mut out,
            };
            scan.visit_block(&method.block);
        }
        out
    }
}

struct CommitmentScan<'a> {
    fn_name: String,
    compound_locals: HashSet<String>,
    out: &'a mut Vec<Finding>,
}

impl<'ast> Visit<'ast> for CommitmentScan<'_> {
    fn visit_expr_method_call(&mut self, i: &'ast ExprMethodCall) {
        if is_sha256_call(i) && self.has_weak_argument(i) {
            let line = i.span().start().line;
            self.out.push(Finding {
                check_name: CHECK_NAME.to_string(),
                severity: Severity::Medium,
                file_path: String::new(),
                line,
                function_name: self.fn_name.clone(),
                description: format!(
                    "Method `{}` uses `env.crypto().sha256()` with a simple argument. \
                     Without a random nonce in the preimage, this is vulnerable to preimage \
                     attacks. Include a nonce in the hash input.",
                    self.fn_name
                ),
            });
        }
        visit::visit_expr_method_call(self, i);
    }
}

fn is_sha256_call(m: &ExprMethodCall) -> bool {
    if m.method != "sha256" {
        return false;
    }
    // Check if receiver is crypto() call
    if let Expr::MethodCall(inner) = &*m.receiver {
        if inner.method == "crypto" {
            return true;
        }
    }
    false
}

/// Locals bound to a compound expression, i.e. a preimage that already combines
/// several values (`let combined = (data, nonce);`).
fn compound_locals(block: &syn::Block) -> HashSet<String> {
    let mut out = HashSet::new();
    for stmt in &block.stmts {
        let syn::Stmt::Local(local) = stmt else {
            continue;
        };
        let Some(init) = &local.init else { continue };
        let compound = matches!(
            &*init.expr,
            Expr::Tuple(_)
                | Expr::Array(_)
                | Expr::Binary(_)
                | Expr::Call(_)
                | Expr::MethodCall(_)
                | Expr::Macro(_)
        );
        if compound {
            if let Some(name) = binding_ident(&local.pat) {
                out.insert(name);
            }
        }
    }
    out
}

impl CommitmentScan<'_> {
    /// Weak when the single preimage argument is a bare literal or a bare name that
    /// was never combined with anything else.
    fn has_weak_argument(&self, m: &ExprMethodCall) -> bool {
        if m.args.len() != 1 {
            return false;
        }
        self.is_weak_preimage(&m.args[0])
    }

    fn is_weak_preimage(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Reference(r) => self.is_weak_preimage(&r.expr),
            Expr::Lit(_) => true,
            Expr::Path(p) => p
                .path
                .get_ident()
                .is_some_and(|id| !self.compound_locals.contains(&id.to_string())),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_file;

    #[test]
    fn flags_sha256_with_simple_arg() {
        let code = r#"
#[contractimpl]
impl MyContract {
    pub fn commit(env: Env, data: Bytes) {
        let hash = env.crypto().sha256(&data);
    }
}
        "#;
        let file = parse_file(code).unwrap();
        let check = WeakCommitmentCheck;
        let findings = check.run(&file, code);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].check_name, CHECK_NAME);
        assert_eq!(findings[0].severity, Severity::Medium);
    }

    #[test]
    fn allows_sha256_with_compound_arg() {
        let code = r#"
#[contractimpl]
impl MyContract {
    pub fn commit(env: Env, data: Bytes, nonce: Bytes) {
        let combined = (data, nonce);
        let hash = env.crypto().sha256(&combined);
    }
}
        "#;
        let file = parse_file(code).unwrap();
        let check = WeakCommitmentCheck;
        let findings = check.run(&file, code);
        assert!(findings.is_empty());
    }

    #[test]
    fn flags_sha256_with_literal() {
        let code = r#"
#[contractimpl]
impl MyContract {
    pub fn commit(env: Env) {
        let hash = env.crypto().sha256(&b"fixed");
    }
}
        "#;
        let file = parse_file(code).unwrap();
        let check = WeakCommitmentCheck;
        let findings = check.run(&file, code);
        assert_eq!(findings.len(), 1);
    }
}
