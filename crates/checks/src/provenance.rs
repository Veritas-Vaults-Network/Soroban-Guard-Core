//! Shared provenance-tracing infrastructure for value-origin analysis.
//!
//! Tracks how values flow through `let` bindings, helper function return values,
//! and method call chains to determine whether an expression ultimately originates
//! from a literal constant, a `.min()` / `.clamp()` call, a function parameter,
//! or is untraceable.

use std::collections::HashMap;
use syn::{Block, Expr, ExprMethodCall, File, FnArg, Item, Pat, Stmt};

/// How far the tracer will follow through helper-function return values.
pub(crate) const MAX_HOPS: usize = 3;

/// The ultimate origin of a value after tracing through bindings and calls.
#[derive(Clone, PartialEq, Eq)]
pub enum Origin {
    /// The value is (or traces to) a literal constant.
    Literal,
    /// The value has a `.min()` or `.clamp()` call somewhere on its provenance chain.
    Clamped,
    /// The value traces back to a function parameter (caller-controlled, unclamped).
    Parameter(String),
    /// The origin cannot be determined within the hop limit or due to complex expressions.
    Untraceable,
}

impl std::fmt::Debug for Origin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Origin::Literal => write!(f, "Literal"),
            Origin::Clamped => write!(f, "Clamped"),
            Origin::Parameter(name) => write!(f, "Parameter({name})"),
            Origin::Untraceable => write!(f, "Untraceable"),
        }
    }
}

/// A map from variable name to source expression for `let` bindings within a single
/// function body, plus whether that variable was marked as clamped.
#[derive(Clone, Default)]
pub struct BindingMap {
    bindings: HashMap<String, Expr>,
    clamped: HashMap<String, bool>,
}

impl BindingMap {
    /// Build from a function body's block.
    pub fn from_block(block: &Block) -> Self {
        let mut m = Self::default();
        for stmt in &block.stmts {
            if let Stmt::Local(local) = stmt {
                if let Pat::Ident(pat_ident) = &local.pat {
                    let name = pat_ident.ident.to_string();
                    let is_clamped = local
                        .init
                        .as_ref()
                        .map(|i| expr_has_min_or_clamp(&i.expr))
                        .unwrap_or(false);
                    m.clamped.insert(name.clone(), is_clamped);
                    if let Some(init) = &local.init {
                        m.bindings.insert(name, *init.expr.clone());
                    }
                }
            }
        }
        m
    }
}

/// Collect parameter names from a function signature.
pub fn collect_params(sig: &syn::Signature) -> Vec<String> {
    sig.inputs
        .iter()
        .filter_map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                if let Pat::Ident(pat_ident) = &*pat_type.pat {
                    return Some(pat_ident.ident.to_string());
                }
            }
            None
        })
        .collect()
}

/// A map from function name to (param names, body block) for all functions in the file.
#[derive(Clone, Default)]
pub struct FunctionMap {
    functions: HashMap<String, (Vec<String>, Block)>,
}

impl FunctionMap {
    pub fn from_file(file: &File) -> Self {
        let mut m = Self::default();
        for item in &file.items {
            match item {
                Item::Fn(item_fn) => {
                    let name = item_fn.sig.ident.to_string();
                    let params = collect_params(&item_fn.sig);
                    m.functions.insert(name, (params, *item_fn.block.clone()));
                }
                Item::Impl(item_impl) => {
                    for impl_item in &item_impl.items {
                        if let syn::ImplItem::Fn(method) = impl_item {
                            let name = method.sig.ident.to_string();
                            let params = collect_params(&method.sig);
                            m.functions.insert(name, (params, method.block.clone()));
                        }
                    }
                }
                _ => {}
            }
        }
        m
    }

    pub fn get(&self, name: &str) -> Option<&(Vec<String>, Block)> {
        self.functions.get(name)
    }
}

/// Trace an expression to its ultimate origin within a single function scope.
///
/// - `params` is the set of parameter names of the **current** (outer) function.
///   If we trace back to one of these, it is a caller-controlled parameter.
/// - `bindings` maps local variable names to their init expressions.
/// - `fmap` allows following helper-function return values.
/// - `subs` maps parameter names to caller-supplied expressions (for cross-function
///   substitution when tracing into helpers). Pass `&HashMap::new()` from the top level.
/// - `hop_limit` prevents infinite recursion (start with `MAX_HOPS`).
pub fn trace_origin(
    expr: &Expr,
    params: &[String],
    bindings: &BindingMap,
    fmap: &FunctionMap,
    subs: &HashMap<String, Expr>,
    hop_limit: usize,
) -> Origin {
    if hop_limit == 0 {
        return Origin::Untraceable;
    }

    match expr {
        Expr::Lit(_) => Origin::Literal,

        Expr::Path(p) => {
            if let Some(ident) = p.path.get_ident() {
                let name = ident.to_string();

                if bindings.clamped.get(&name).copied().unwrap_or(false) {
                    return Origin::Clamped;
                }

                // Check substitutions first (cross-function parameter mapping)
                if let Some(sub_expr) = subs.get(&name) {
                    return trace_origin(sub_expr, params, bindings, fmap, subs, hop_limit);
                }

                if params.iter().any(|p| p == &name) {
                    return Origin::Parameter(name);
                }

                if let Some(binding_expr) = bindings.bindings.get(&name) {
                    return trace_origin(binding_expr, params, bindings, fmap, subs, hop_limit - 1);
                }

                Origin::Untraceable
            } else {
                Origin::Untraceable
            }
        }

        Expr::MethodCall(m) => {
            if has_min_or_clamp_in_chain(m) {
                return Origin::Clamped;
            }
            trace_origin(&m.receiver, params, bindings, fmap, subs, hop_limit - 1)
        }

        Expr::Call(c) => {
            if let Expr::Path(p) = &*c.func {
                if let Some(ident) = p.path.get_ident() {
                    if let Some((call_params, body)) = fmap.get(&ident.to_string()) {
                        // Build substitution map: helper param → caller arg
                        let mut inner_subs = subs.clone();
                        for (i, cp) in call_params.iter().enumerate() {
                            if let Some(arg) = c.args.iter().nth(i) {
                                inner_subs.insert(cp.clone(), arg.clone());
                            }
                        }
                        if let Some(ret_expr) = return_expr(body) {
                            return trace_origin(
                                ret_expr,
                                params,
                                bindings,
                                fmap,
                                &inner_subs,
                                hop_limit - 1,
                            );
                        }
                    }
                }
            }
            Origin::Untraceable
        }

        _ => Origin::Untraceable,
    }
}

/// Extract the tail expression (implicit return) from a block, if any.
fn return_expr(block: &Block) -> Option<&Expr> {
    match block.stmts.last() {
        Some(Stmt::Expr(expr, _)) => Some(expr),
        _ => None,
    }
}

/// Check if an expression contains a `.min(...)` or `.clamp(...)` method call
/// anywhere in the tree (used to mark let bindings as clamped).
fn expr_has_min_or_clamp(expr: &Expr) -> bool {
    match expr {
        Expr::MethodCall(m) => {
            let name = m.method.to_string();
            name == "min" || name == "clamp" || expr_has_min_or_clamp(&m.receiver)
        }
        Expr::Binary(b) => expr_has_min_or_clamp(&b.left) || expr_has_min_or_clamp(&b.right),
        _ => false,
    }
}

/// Check if the immediate receiver chain of a method call contains `.min()` or `.clamp()`.
fn has_min_or_clamp_in_chain(m: &ExprMethodCall) -> bool {
    let name = m.method.to_string();
    if name == "min" || name == "clamp" {
        return true;
    }
    match &*m.receiver {
        Expr::MethodCall(inner) => has_min_or_clamp_in_chain(inner),
        Expr::Path(_) => false,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_file;

    fn mk_binding_map(src: &str) -> (BindingMap, Vec<String>) {
        let file = parse_file(src).unwrap();
        for item in &file.items {
            if let Item::Fn(f) = item {
                let params = collect_params(&f.sig);
                let bindings = BindingMap::from_block(&f.block);
                return (bindings, params);
            }
        }
        panic!("no function found in source");
    }

    #[test]
    fn traces_literal() {
        let file = parse_file(
            r#"
fn foo() {
    let x = 42;
}
"#,
        )
        .unwrap();
        let fmap = FunctionMap::from_file(&file);
        let (bindings, params) = mk_binding_map("fn foo() { let x = 42; }");
        let expr: Expr = syn::parse_str("x").unwrap();
        let origin = trace_origin(&expr, &params, &bindings, &fmap, &HashMap::new(), MAX_HOPS);
        assert_eq!(origin, Origin::Literal);
    }

    #[test]
    fn traces_parameter() {
        let (bindings, params) = mk_binding_map("fn foo(ttl: u32) { let x = ttl; }");
        let fmap = FunctionMap::default();
        let expr: Expr = syn::parse_str("x").unwrap();
        let origin = trace_origin(&expr, &params, &bindings, &fmap, &HashMap::new(), MAX_HOPS);
        assert_eq!(origin, Origin::Parameter("ttl".to_string()));
    }

    #[test]
    fn traces_clamped_binding() {
        let (bindings, params) = mk_binding_map("fn foo(ttl: u32) { let capped = ttl.min(1000); }");
        let fmap = FunctionMap::default();
        let expr: Expr = syn::parse_str("capped").unwrap();
        let origin = trace_origin(&expr, &params, &bindings, &fmap, &HashMap::new(), MAX_HOPS);
        assert_eq!(origin, Origin::Clamped);
    }

    #[test]
    fn traces_method_call_with_min() {
        let (bindings, params) = mk_binding_map("fn foo() {}");
        let fmap = FunctionMap::default();
        let expr: Expr = syn::parse_str("ttl.min(1000)").unwrap();
        let origin = trace_origin(&expr, &params, &bindings, &fmap, &HashMap::new(), MAX_HOPS);
        assert_eq!(origin, Origin::Clamped);
    }

    #[test]
    fn traces_method_call_with_clamp() {
        let (bindings, params) = mk_binding_map("fn foo() {}");
        let fmap = FunctionMap::default();
        let expr: Expr = syn::parse_str("ttl.clamp(0, 1000)").unwrap();
        let origin = trace_origin(&expr, &params, &bindings, &fmap, &HashMap::new(), MAX_HOPS);
        assert_eq!(origin, Origin::Clamped);
    }
}
