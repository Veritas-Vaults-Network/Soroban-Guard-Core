use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ExprMethodCall, File, ImplItem, ItemImpl};

const CHECK_NAME: &str = "interprocedural-storage-toctou";

pub struct InterproceduralStorageTocTouCheck;

fn get_storage_tier(m: &ExprMethodCall) -> Option<String> {
    let mut current = &m.receiver;
    loop {
        match &**current {
            Expr::MethodCall(mc) => {
                if matches!(
                    mc.method.to_string().as_str(),
                    "persistent" | "instance" | "temporary"
                ) {
                    return Some(mc.method.to_string());
                }
                current = &mc.receiver;
            }
            _ => return None,
        }
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

fn expr_to_string(expr: &Expr) -> String {
    expr.to_token_stream().to_string()
}

struct StorageOp {
    kind: OpKind,
    key_tokens: String,
    tier: String,
    line: usize,
    function_name: String,
}

#[derive(Clone, Copy, PartialEq)]
enum OpKind {
    Read,
    Write,
}

fn method_name_is_storage_read(name: &str) -> bool {
    matches!(name, "has" | "get")
}

fn method_name_is_storage_write(name: &str) -> bool {
    matches!(name, "set" | "remove")
}

fn collect_ops_in_block<'a>(
    block: &'a syn::Block,
    current_fn: &str,
    impl_items: &'a [ImplItem],
) -> Vec<StorageOp> {
    let mut ops = Vec::new();
    collect_ops_visitor(block, current_fn, impl_items, &mut ops);
    ops
}

fn collect_ops_visitor(
    block: &syn::Block,
    current_fn: &str,
    impl_items: &[ImplItem],
    ops: &mut Vec<StorageOp>,
) {
    struct Inner<'a> {
        current_fn: &'a str,
        impl_items: &'a [ImplItem],
        ops: &'a mut Vec<StorageOp>,
    }

    impl<'ast> Visit<'ast> for Inner<'ast> {
        fn visit_expr_method_call(&mut self, i: &'ast ExprMethodCall) {
            let method_name = i.method.to_string();

            if method_name_is_storage_read(&method_name) || method_name_is_storage_write(&method_name)
            {
                if receiver_chain_contains_storage(&i.receiver) {
                    if let Some(tier) = get_storage_tier(i) {
                        if let Some(arg) = i.args.first() {
                            let kind = if method_name_is_storage_read(&method_name) {
                                OpKind::Read
                            } else {
                                OpKind::Write
                            };
                            self.ops.push(StorageOp {
                                kind,
                                key_tokens: expr_to_string(arg),
                                tier,
                                line: i.span().start().line,
                                function_name: self.current_fn.to_string(),
                            });
                        }
                    }
                }
            }

            visit::visit_expr_method_call(self, i);
        }

        fn visit_expr_call(&mut self, i: &'ast syn::ExprCall) {
            if let Expr::Path(p) = &*i.func {
                if let Some(segment) = p.path.segments.last() {
                    let callee = segment.ident.to_string();
                    for item in self.impl_items {
                        if let ImplItem::Fn(m) = item {
                            if m.sig.ident == callee {
                                let mut inner = Inner {
                                    current_fn: &callee,
                                    impl_items: self.impl_items,
                                    ops: &mut *self.ops,
                                };
                                inner.visit_block(&m.block);
                                return;
                            }
                        }
                    }
                }
            }
            visit::visit_expr_call(self, i);
        }
    }

    let mut inner = Inner {
        current_fn,
        impl_items,
        ops,
    };
    inner.visit_block(block);
}

fn is_read_same_fn_as_write(read: &StorageOp, write: &StorageOp) -> bool {
    read.function_name == write.function_name
}

impl Check for InterproceduralStorageTocTouCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();

        let contractimpl_blocks: Vec<&ItemImpl> = file
            .items
            .iter()
            .filter_map(|item| {
                if let syn::Item::Impl(item_impl) = item {
                    if crate::util::is_contractimpl(item_impl) {
                        return Some(item_impl);
                    }
                }
                None
            })
            .collect();

        for impl_block in &contractimpl_blocks {
            for entrypoint in contractimpl_functions(file) {
                let fn_name = entrypoint.sig.ident.to_string();
                let mut reachable_ops =
                    collect_ops_in_block(&entrypoint.block, &fn_name, &impl_block.items);

                let mut own_function_names: Vec<String> = Vec::new();

                struct OwnFnCollector<'a> {
                    own_fns: &'a mut Vec<String>,
                }
                impl<'ast> Visit<'ast> for OwnFnCollector<'ast> {
                    fn visit_expr_call(&mut self, i: &'ast syn::ExprCall) {
                        if let Expr::Path(p) = &*i.func {
                            if let Some(ident) = p.path.get_ident() {
                                self.own_fns.push(ident.to_string());
                            }
                        }
                        visit::visit_expr_call(self, i);
                    }
                }
                let mut collector = OwnFnCollector {
                    own_fns: &mut own_function_names,
                };
                collector.visit_block(&entrypoint.block);

                if reachable_ops.len() < 2 {
                    continue;
                }

                let reads: Vec<&StorageOp> = reachable_ops
                    .iter()
                    .filter(|o| o.kind == OpKind::Read)
                    .collect();
                let writes: Vec<&StorageOp> = reachable_ops
                    .iter()
                    .filter(|o| o.kind == OpKind::Write)
                    .collect();

                for read in &reads {
                    for write in &writes {
                        if read.tier != write.tier {
                            continue;
                        }
                        if read.key_tokens != write.key_tokens {
                            continue;
                        }
                        if is_read_same_fn_as_write(read, write) {
                            continue;
                        }
                        if read.line >= write.line {
                            continue;
                        }

                        out.push(Finding {
                            check_name: CHECK_NAME.to_string(),
                            severity: Severity::High,
                            file_path: String::new(),
                            line: read.line,
                            function_name: fn_name.clone(),
                            description: format!(
                                "Interprocedural TOCTOU: `{}()`/{}/`{}` reads key `{}` in `{}()` at line {} \
                                 but `{}()` in `{}()` writes the same key at line {} without re-checking. \
                                 An attacker can race the read and write across function boundaries.",
                                read.tier,
                                if read.key_tokens.starts_with('&') { "has" } else { "get" },
                                read.key_tokens,
                                read.key_tokens,
                                read.function_name,
                                read.line,
                                write.tier,
                                write.function_name,
                                write.line,
                            ),
                        });
                        break;
                    }
                }
            }
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Check;
    use syn::parse_file;

    fn run_on_src(src: &str) -> Result<Vec<Finding>, syn::Error> {
        let file = parse_file(src)?;
        Ok(InterproceduralStorageTocTouCheck.run(&file, src))
    }

    #[test]
    fn flags_claim_do_claim_split() -> Result<(), syn::Error> {
        let hits = run_on_src(
            r#"
use soroban_sdk::{contractimpl, symbol_short, Env, Symbol};

pub struct C;

const CLAIMED: Symbol = symbol_short!("claimed");

#[contractimpl]
impl C {
    fn already_claimed(env: &Env) -> bool {
        env.storage().persistent().has(&CLAIMED)
    }

    fn do_claim(env: &Env) {
        env.storage().persistent().set(&CLAIMED, &true);
    }

    pub fn claim(env: Env) {
        if !Self::already_claimed(&env) {
            Self::do_claim(&env);
        }
    }
}
"#,
        )?;
        assert_eq!(hits.len(), 1, "should flag the TOCTOU pattern");
        assert_eq!(hits[0].severity, Severity::High);
        assert_eq!(hits[0].check_name, CHECK_NAME);
        Ok(())
    }

    #[test]
    fn safe_when_recheck_in_write_fn() -> Result<(), syn::Error> {
        let hits = run_on_src(
            r#"
use soroban_sdk::{contractimpl, symbol_short, Env, Symbol};

pub struct C;

const CLAIMED: Symbol = symbol_short!("claimed");

#[contractimpl]
impl C {
    fn already_claimed_and_set(env: &Env) -> bool {
        if env.storage().persistent().has(&CLAIMED) {
            return true;
        }
        env.storage().persistent().set(&CLAIMED, &true);
        false
    }

    pub fn claim(env: Env) {
        Self::already_claimed_and_set(&env);
    }
}
"#,
        )?;
        assert!(
            hits.is_empty(),
            "safe pattern should not be flagged: {:?}",
            hits
        );
        Ok(())
    }

    #[test]
    fn safe_no_cross_function_ops() -> Result<(), syn::Error> {
        let hits = run_on_src(
            r#"
use soroban_sdk::{contractimpl, symbol_short, Env, Symbol};

pub struct C;

const K: Symbol = symbol_short!("k");

#[contractimpl]
impl C {
    pub fn get_directly(env: Env) {
        let _val = env.storage().persistent().get(&K);
    }
}
"#,
        )?;
        assert!(hits.is_empty());
        Ok(())
    }

    #[test]
    fn ignores_different_keys() -> Result<(), syn::Error> {
        let hits = run_on_src(
            r#"
use soroban_sdk::{contractimpl, symbol_short, Env, Symbol};

pub struct C;

const K1: Symbol = symbol_short!("k1");
const K2: Symbol = symbol_short!("k2");

#[contractimpl]
impl C {
    fn read_k1(env: &Env) -> bool {
        env.storage().persistent().has(&K1)
    }

    fn write_k2(env: &Env) {
        env.storage().persistent().set(&K2, &true);
    }

    pub fn do_stuff(env: Env) {
        if Self::read_k1(&env) {
            Self::write_k2(&env);
        }
    }
}
"#,
        )?;
        assert!(
            hits.is_empty(),
            "different keys should not be flagged: {:?}",
            hits
        );
        Ok(())
    }

    #[test]
    fn ignores_different_tiers() -> Result<(), syn::Error> {
        let hits = run_on_src(
            r#"
use soroban_sdk::{contractimpl, symbol_short, Env, Symbol};

pub struct C;

const K: Symbol = symbol_short!("k");

#[contractimpl]
impl C {
    fn read_instance(env: &Env) -> bool {
        env.storage().instance().has(&K)
    }

    fn write_persistent(env: &Env) {
        env.storage().persistent().set(&K, &true);
    }

    pub fn do_stuff(env: Env) {
        if Self::read_instance(&env) {
            Self::write_persistent(&env);
        }
    }
}
"#,
        )?;
        assert!(
            hits.is_empty(),
            "different tiers should not be flagged: {:?}",
            hits
        );
        Ok(())
    }
}
