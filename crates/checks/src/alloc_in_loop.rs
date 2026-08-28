//! Detects `Vec::new()` or `Map::new()` allocations inside loops.
//!
//! Allocating a new Vec or Map on every iteration of a loop inside a Soroban
//! contract wastes compute budget and can cause the transaction to exceed
//! resource limits. Collections should be allocated once before the loop.

use crate::util::contractimpl_functions;
use crate::{Check, Finding, Severity};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ExprCall, ExprForLoop, ExprLoop, ExprWhile, File};

const CHECK_NAME: &str = "alloc-in-loop";

/// Flags `Vec::new()` or `Map::new()` calls inside for/while/loop blocks
/// within `#[contractimpl]` functions.
pub struct AllocInLoopCheck;

impl Check for AllocInLoopCheck {
    fn name(&self) -> &str {
        CHECK_NAME
    }

    fn run(&self, file: &File, _source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        for method in contractimpl_functions(file) {
            let fn_name = method.sig.ident.to_string();
            let mut scan = AllocScan {
                fn_name,
                loop_depth: 0,
                out: &mut out,
            };
            scan.visit_block(&method.block);
        }
        out
    }
}

struct AllocScan<'a> {
    fn_name: String,
    loop_depth: usize,
    out: &'a mut Vec<Finding>,
}

impl<'ast> Visit<'ast> for AllocScan<'ast> {
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

    fn visit_expr_call(&mut self, i: &'ast ExprCall) {
        if self.loop_depth > 0 && is_vec_or_map_new(i) {
            let line = i.span().start().line;
            let alloc_type = alloc_type_name(i);
            self.out.push(Finding {
                check_name: CHECK_NAME.to_string(),
                severity: Severity::Low,
                file_path: String::new(),
                line,
                function_name: self.fn_name.clone(),
                description: format!(
                    "Method `{}` allocates `{}::new()` inside a loop. This wastes compute \
                     budget on every iteration. Allocate the collection once before the loop.",
                    self.fn_name, alloc_type
                ),
            });
        }
        visit::visit_expr_call(self, i);
    }
}

/// Returns true for `Vec::new(...)` and `Map::new(...)` call expressions.
fn is_vec_or_map_new(expr: &ExprCall) -> bool {
    if let Expr::Path(p) = &*expr.func {
        let segs: Vec<_> = p.path.segments.iter().collect();
        // Match two-segment paths like `Vec::new` or `Map::new`
        if segs.len() == 2
            && matches!(segs[0].ident.to_string().as_str(), "Vec" | "Map")
            && segs[1].ident == "new"
        {
            return true;
        }
    }
    false
}

fn alloc_type_name(expr: &ExprCall) -> &'static str {
    if let Expr::Path(p) = &*expr.func {
        if let Some(seg) = p.path.segments.iter().next() {
            return match seg.ident.to_string().as_str() {
                "Vec" => "Vec",
                "Map" => "Map",
                _ => "Collection",
            };
        }
    }
    "Collection"
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_file;

    #[test]
    fn detects_vec_new_in_for_loop() {
        let code = r#"
#[contractimpl]
impl MyContract {
    pub fn process(env: Env) {
        for i in 0..10 {
            let v = Vec::new(&env);
        }
    }
}
        "#;
        let file = parse_file(code).unwrap();
        let findings = AllocInLoopCheck.run(&file, code);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].check_name, CHECK_NAME);
        assert_eq!(findings[0].severity, Severity::Low);
    }

    #[test]
    fn detects_map_new_in_while_loop() {
        let code = r#"
#[contractimpl]
impl MyContract {
    pub fn process(env: Env) {
        let mut i = 0;
        while i < 10 {
            let m = Map::new(&env);
            i += 1;
        }
    }
}
        "#;
        let file = parse_file(code).unwrap();
        let findings = AllocInLoopCheck.run(&file, code);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].check_name, CHECK_NAME);
    }

    #[test]
    fn allows_alloc_outside_loop() {
        let code = r#"
#[contractimpl]
impl MyContract {
    pub fn process(env: Env) {
        let v = Vec::new(&env);
        for i in 0..10 {
            v.push_back(i);
        }
    }
}
        "#;
        let file = parse_file(code).unwrap();
        let findings = AllocInLoopCheck.run(&file, code);
        assert!(findings.is_empty());
    }
}
