//! Missing `env.events().publish()` when deploying a sub-contract via `env.deployer().deploy(...)`
//! in `#[contractimpl]` methods.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ExprMethodCall, File};

const CHECK_NAME: &str = "deploy-no-event";

/// Flags `#[contractimpl]` methods that call `env.deployer().deploy(...)` without
/// calling `env.events().publish()` to emit an event for the deployment.
pub struct DeployNoEventCheck;

impl Check for DeployNoEventCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        for method in contractimpl_functions(file) {
            let mut scan = FuncBodyScan::default();
            scan.visit_block(&method.block);
            if !scan.deploy_found || scan.events_publish {
                continue;
            }
            let line = scan.deploy_line.unwrap_or_else(|| method.sig.ident.span().start().line);
            let fn_name = method.sig.ident.to_string();
            out.push(Finding {
                check_name: CHECK_NAME.to_string(),
                severity: Severity::Low, // as per issue
                file_path: String::new(),
                line,
                function_name: fn_name.clone(),
                description: format!(
                    "Method `{fn_name}` calls `env.deployer().deploy(...)` but does not call \
                     `env.events().publish()`. Off-chain indexers and users cannot track \
                     contract deployment activity without events."
                ),
            });
        }
        out
    }
}

fn is_deployer_deploy(m: &ExprMethodCall) -> bool {
    if m.method != "deploy" && m.method != "deploy_v2" {
        return false;
    }
    // receiver must be `env.deployer()` or a chain containing it
    receiver_contains_deployer(&m.receiver)
}

fn receiver_contains_deployer(expr: &Expr) -> bool {
    match expr {
        Expr::MethodCall(m) => m.method == "deployer" || receiver_contains_deployer(&m.receiver),
        _ => false,
    }
}

fn is_events_publish(m: &ExprMethodCall) -> bool {
    if m.method != "publish" {
        return false;
    }
    // Check if the receiver chain contains events()
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

#[derive(Default)]
struct FuncBodyScan {
    deploy_found: bool,
    events_publish: bool,
    deploy_line: Option<usize>,
}

impl<'ast> Visit<'ast> for FuncBodyScan {
    fn visit_expr_method_call(&mut self, i: &'ast ExprMethodCall) {
        if is_deployer_deploy(i) && !self.deploy_found {
            self.deploy_found = true;
            self.deploy_line = Some(i.span().start().line);
        }
        if is_events_publish(i) {
            self.events_publish = true;
        }
        visit::visit_expr_method_call(self, i);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Check;
    use syn::parse_file;

    fn run_on_src(src: &str) -> Result<Vec<Finding>, syn::Error> {
        let file = parse_file(src)?;
        Ok(DeployNoEventCheck.run(&file, src))
    }

    #[test]
    fn flags_deploy_without_events_publish() -> Result<(), syn::Error> {
        let hits = run_on_src(
            r#"
use soroban_sdk::{contractimpl, Env, BytesN};

pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn deploy_sub(env: Env, wasm_hash: BytesN<32>, salt: BytesN<32>) {
        env.deployer().deploy(wasm_hash, salt);
    }
}
"#,
        )?;
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].function_name, "deploy_sub");
        assert_eq!(hits[0].severity, Severity::Low);
        assert_eq!(hits[0].check_name, CHECK_NAME);
        Ok(())
    }

    #[test]
    fn does_not_flag_deploy_with_events_publish() -> Result<(), syn::Error> {
        let hits = run_on_src(
            r#"
use soroban_sdk::{contractimpl, Env, BytesN, symbol_short};

pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn deploy_sub(env: Env, wasm_hash: BytesN<32>, salt: BytesN<32>) {
        env.deployer().deploy(wasm_hash, salt);
        env.events().publish(("deployed",), (wasm_hash, salt));
    }
}
"#,
        )?;
        assert_eq!(hits.len(), 0);
        Ok(())
    }

    #[test]
    fn flags_multiple_deploys_without_events() -> Result<(), syn::Error> {
        let hits = run_on_src(
            r#"
use soroban_sdk::{contractimpl, Env, BytesN};

pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn deploy_two(env: Env, wasm_hash1: BytesN<32>, wasm_hash2: BytesN<32>, salt: BytesN<32>) {
        env.deployer().deploy(wasm_hash1, salt);
        env.deployer().deploy(wasm_hash2, salt);
    }
}
"#,
        )?;
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].function_name, "deploy_two");
        Ok(())
    }

    #[test]
    fn does_not_flag_read_only_methods() -> Result<(), syn::Error> {
        let hits = run_on_src(
            r#"
use soroban_sdk::{contractimpl, Env};

pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn get() -> u32 {
        1
    }
}
"#,
        )?;
        assert_eq!(hits.len(), 0);
        Ok(())
    }
}
