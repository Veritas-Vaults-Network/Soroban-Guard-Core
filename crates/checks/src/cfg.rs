//! Minimal control-flow graph + dominance analysis, shared by checks that need
//! to answer an ordering question ("does A always happen before B on every
//! path?") rather than "does A appear somewhere in the function?".
//!
//! This is intentionally not a general-purpose compiler CFG. It models just
//! enough of Rust's structured control flow to build a sound graph from a
//! `syn::Block`:
//!
//! - straight-line statement sequences,
//! - `if`/`else` (including `if` with no `else`, treated as an implicit empty
//!   `else` that jumps straight to the join point),
//! - `match` (every arm is a branch into the join point),
//! - `while` / `for` (the loop header has an edge into the body *and* an edge
//!   that skips straight to the code after the loop, since the condition may
//!   be false on entry),
//! - `loop` (no skip edge — the body always executes at least once; the only
//!   way out is an explicit `break`),
//! - `return`, `break`, `continue` as hard terminators that cut off
//!   fall-through within the current sequence,
//! - calls to other functions in the same file: a whole-statement call to a
//!   function present in the supplied registry is *inlined* — the callee's
//!   own CFG is spliced in at the call site — so markers found inside a
//!   helper participate in the caller's dominance graph. Direct/mutual
//!   recursion is broken by a call-stack guard (a function already being
//!   expanded on the current path is treated as opaque).
//!
//! What it does *not* model: closures, `?`-early-return (deliberately: the
//! two paths through a `?` are "continue normally" or "leave the function
//! entirely", neither of which changes whether an earlier write dominates a
//! later call site, so it can be treated as a plain expression), or branch
//! conditions nested inside arbitrary sub-expressions (only `if`/`match`/loop
//! used as a whole statement or as a `let` initializer are split into CFG
//! nodes — the same shapes idiomatic Soroban contracts use).
//!
//! Consumers classify expressions into "markers" via a caller-supplied
//! closure returning a small tag (`u8`); the CFG records where each marker
//! occurs (block + a monotonically increasing order key) and can then answer
//! "does the block containing marker A dominate the block containing marker
//! B, with A occurring first when they share a block?".

use std::collections::{HashMap, HashSet};
use syn::{Block, Expr, ExprCall, ExprMethodCall, Local, Stmt};

pub type BlockId = usize;

/// A single classified occurrence of interest inside the function (or an
/// inlined helper), in the order the CFG builder encountered it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Marker {
    pub tag: u8,
    pub order: usize,
    pub line: usize,
}

struct BlockData {
    markers: Vec<Marker>,
    succs: Vec<BlockId>,
}

/// A built control-flow graph plus every marker the classifier found while
/// walking it (with inlining already applied).
pub struct Cfg {
    blocks: Vec<BlockData>,
    entry: BlockId,
}

#[derive(Clone, Copy)]
struct LoopCtx {
    header: BlockId,
    after: BlockId,
}

struct Builder<'a> {
    blocks: Vec<BlockData>,
    registry: &'a HashMap<String, &'a Block>,
    classify: &'a dyn Fn(&Expr) -> Option<u8>,
    expanding: Vec<String>,
    order: usize,
    loops: Vec<LoopCtx>,
}

impl<'a> Builder<'a> {
    fn new_block(&mut self) -> BlockId {
        self.blocks.push(BlockData {
            markers: Vec::new(),
            succs: Vec::new(),
        });
        self.blocks.len() - 1
    }

    fn add_edge(&mut self, from: BlockId, to: BlockId) {
        self.blocks[from].succs.push(to);
    }

    fn next_order(&mut self) -> usize {
        let o = self.order;
        self.order += 1;
        o
    }

    /// Scan a leaf statement/expression for markers, in AST pre-order.
    fn scan_leaf_expr(&mut self, block: BlockId, expr: &Expr) {
        let mut v = MarkerVisitor {
            builder: self,
            block,
        };
        v.walk_expr(expr);
    }

    fn add_marker(&mut self, block: BlockId, tag: u8, line: usize) {
        let order = self.next_order();
        self.blocks[block].markers.push(Marker { tag, order, line });
    }

    /// Try to resolve a whole-statement call expression to a local function
    /// name. Handles bare calls (`foo(..)`), `Self::foo(..)` / `Type::foo(..)`,
    /// and `recv.foo(..)` method calls.
    fn resolve_call_name(expr: &Expr) -> Option<String> {
        match expr {
            Expr::Call(ExprCall { func, .. }) => match &**func {
                Expr::Path(p) => p.path.segments.last().map(|s| s.ident.to_string()),
                _ => None,
            },
            Expr::MethodCall(ExprMethodCall { method, .. }) => Some(method.to_string()),
            _ => None,
        }
    }

    /// If `expr` is a whole-statement call to a registry function (and we are
    /// not already expanding it on this path), splice the callee's CFG in
    /// between `current` and a fresh continuation block, returning that
    /// continuation. Returns `None` if this expression isn't an inlinable
    /// call (caller should fall back to leaf scanning).
    fn try_inline(&mut self, current: BlockId, expr: &Expr) -> Option<BlockId> {
        let name = Self::resolve_call_name(expr)?;
        let callee = *self.registry.get(&name)?;
        if self.expanding.contains(&name) {
            return None;
        }
        // Scan the call's own receiver/args for markers before descending,
        // since e.g. the receiver chain of a method call can itself contain
        // a classified expression.
        self.scan_leaf_expr(current, expr);

        self.expanding.push(name);
        let callee_entry = self.new_block();
        self.add_edge(current, callee_entry);
        let exit = self.build_seq(&callee.stmts, callee_entry);
        self.expanding.pop();

        match exit {
            Some(exit_block) => {
                let cont = self.new_block();
                self.add_edge(exit_block, cont);
                Some(cont)
            }
            // The callee never falls through (e.g. it always returns/panics
            // on every path) — nothing after this call site is reachable via
            // this path.
            None => None,
        }
    }

    /// Build a straight-line/branching sequence starting at `start`. Returns
    /// `Some(open_block)` if control can fall off the end of the sequence, or
    /// `None` if every path through it terminates (return/break/continue, or
    /// an inlined call that never returns).
    fn build_seq(&mut self, stmts: &[Stmt], start: BlockId) -> Option<BlockId> {
        let mut current = start;
        for stmt in stmts {
            match stmt {
                Stmt::Expr(expr, _) => {
                    if let Some(next) = self.build_expr_stmt(current, expr)? {
                        current = next;
                    }
                }
                Stmt::Local(local) => {
                    current = self.build_local(current, local)?;
                }
                Stmt::Macro(m) => {
                    self.scan_macro(current, &m.mac);
                }
                Stmt::Item(_) => {}
            }
        }
        Some(current)
    }

    /// Returns `Ok`-shaped `Option<Option<BlockId>>`-ish: outer `None` means
    /// "terminated, stop processing the sequence"; inner `None` means "this
    /// statement had no control effect, keep `current` as-is" (only used via
    /// `?` on the outer level — see call site).
    fn build_expr_stmt(&mut self, current: BlockId, expr: &Expr) -> Option<Option<BlockId>> {
        match expr {
            Expr::If(e) => {
                let join = self.new_block();
                self.build_if(e, current, join);
                Some(Some(join))
            }
            Expr::Match(e) => {
                let join = self.new_block();
                self.build_match(e, current, join);
                Some(Some(join))
            }
            Expr::While(e) => {
                let after = self.build_while_like(&e.cond, &e.body, current);
                Some(Some(after))
            }
            Expr::ForLoop(e) => {
                let after = self.build_while_like_forloop(&e.body, current);
                Some(Some(after))
            }
            Expr::Loop(e) => {
                let after = self.build_plain_loop(&e.body, current);
                Some(Some(after))
            }
            Expr::Return(_) => None,
            Expr::Break(b) => {
                if let Some(ctx) = self.loops.last().copied() {
                    self.add_edge(current, ctx.after);
                }
                let _ = b;
                None
            }
            Expr::Continue(_) => {
                if let Some(ctx) = self.loops.last().copied() {
                    self.add_edge(current, ctx.header);
                }
                None
            }
            Expr::Block(b) => {
                let inner = self.build_seq(&b.block.stmts, current);
                Some(inner)
            }
            _ => {
                if let Some(cont) = self.try_inline(current, expr) {
                    Some(Some(cont))
                } else {
                    self.scan_leaf_expr(current, expr);
                    Some(Some(current))
                }
            }
        }
    }

    fn build_local(&mut self, current: BlockId, local: &Local) -> Option<BlockId> {
        let Some(init) = &local.init else {
            return Some(current);
        };
        // `let x = if/match/loop { ... };` — split on the initializer.
        match self.build_expr_stmt(current, &init.expr) {
            Some(Some(next)) => Some(next),
            Some(None) => Some(current),
            None => None,
        }
    }

    fn build_if(&mut self, e: &syn::ExprIf, from: BlockId, join: BlockId) {
        let then_entry = self.new_block();
        self.add_edge(from, then_entry);
        if let Some(exit) = self.build_seq(&e.then_branch.stmts, then_entry) {
            self.add_edge(exit, join);
        }

        match &e.else_branch {
            Some((_, else_expr)) => {
                let else_entry = self.new_block();
                self.add_edge(from, else_entry);
                match self.build_expr_stmt(else_entry, else_expr) {
                    Some(Some(exit)) => self.add_edge(exit, join),
                    Some(None) | None => {}
                }
            }
            None => {
                // No else: falling through the condition-false path goes
                // straight to the join point.
                self.add_edge(from, join);
            }
        }
    }

    fn build_match(&mut self, e: &syn::ExprMatch, from: BlockId, join: BlockId) {
        for arm in &e.arms {
            let arm_entry = self.new_block();
            self.add_edge(from, arm_entry);
            match self.build_expr_stmt(arm_entry, &arm.body) {
                Some(Some(exit)) => self.add_edge(exit, join),
                Some(None) | None => {}
            }
        }
    }

    fn build_while_like(&mut self, _cond: &Expr, body: &Block, from: BlockId) -> BlockId {
        let header = self.new_block();
        self.add_edge(from, header);
        let after = self.new_block();
        // Condition may be false immediately: skip the body entirely.
        self.add_edge(header, after);

        let body_entry = self.new_block();
        self.add_edge(header, body_entry);
        self.loops.push(LoopCtx { header, after });
        if let Some(exit) = self.build_seq(&body.stmts, body_entry) {
            self.add_edge(exit, header);
        }
        self.loops.pop();
        after
    }

    fn build_while_like_forloop(&mut self, body: &Block, from: BlockId) -> BlockId {
        self.build_while_like(&syn::parse_quote!(true), body, from)
    }

    fn build_plain_loop(&mut self, body: &Block, from: BlockId) -> BlockId {
        let header = self.new_block();
        self.add_edge(from, header);
        let after = self.new_block();

        let body_entry = self.new_block();
        self.add_edge(header, body_entry);
        self.loops.push(LoopCtx { header, after });
        if let Some(exit) = self.build_seq(&body.stmts, body_entry) {
            self.add_edge(exit, header);
        }
        self.loops.pop();
        after
    }

    fn scan_macro(&mut self, block: BlockId, mac: &syn::Macro) {
        if let Ok(expr) = mac.parse_body::<Expr>() {
            self.scan_leaf_expr(block, &expr);
        }
    }
}

/// Walks an expression tree in pre-order looking for classified markers and
/// inlinable calls to registry functions that appear as sub-expressions
/// rather than whole statements (e.g. inside a `let` initializer that is
/// itself a call: `let x = self.helper(y);`).
struct MarkerVisitor<'a, 'b> {
    builder: &'a mut Builder<'b>,
    block: BlockId,
}

impl<'a, 'b> MarkerVisitor<'a, 'b> {
    fn walk_expr(&mut self, expr: &Expr) {
        if let Some(tag) = (self.builder.classify)(expr) {
            let line = line_of(expr);
            self.builder.add_marker(self.block, tag, line);
        }
        match expr {
            Expr::Call(c) => {
                self.walk_expr(&c.func);
                for a in &c.args {
                    self.walk_expr(a);
                }
            }
            Expr::MethodCall(m) => {
                self.walk_expr(&m.receiver);
                for a in &m.args {
                    self.walk_expr(a);
                }
            }
            Expr::Field(f) => self.walk_expr(&f.base),
            Expr::Reference(r) => self.walk_expr(&r.expr),
            Expr::Paren(p) => self.walk_expr(&p.expr),
            Expr::Unary(u) => self.walk_expr(&u.expr),
            Expr::Binary(b) => {
                self.walk_expr(&b.left);
                self.walk_expr(&b.right);
            }
            Expr::Assign(a) => {
                self.walk_expr(&a.left);
                self.walk_expr(&a.right);
            }
            Expr::Try(t) => self.walk_expr(&t.expr),
            Expr::Cast(c) => self.walk_expr(&c.expr),
            Expr::Tuple(t) => {
                for e in &t.elems {
                    self.walk_expr(e);
                }
            }
            _ => {}
        }
    }
}

fn line_of(expr: &Expr) -> usize {
    use syn::spanned::Spanned;
    expr.span().start().line
}

/// Build the CFG for `body`, inlining whole-statement calls to any function
/// present in `registry`. `classify` tags expressions of interest; markers
/// are recorded with the block they land in and a global order key that
/// approximates execution order (including inside inlined callees).
pub fn build(
    body: &Block,
    registry: &HashMap<String, &Block>,
    classify: &dyn Fn(&Expr) -> Option<u8>,
) -> Cfg {
    let mut builder = Builder {
        blocks: Vec::new(),
        registry,
        classify,
        expanding: Vec::new(),
        order: 0,
        loops: Vec::new(),
    };
    let entry = builder.new_block();
    builder.build_seq(&body.stmts, entry);
    Cfg {
        blocks: builder.blocks,
        entry,
    }
}

impl Cfg {
    /// Every marker with the given tag, paired with the block it occurs in.
    pub fn markers_with_tag(&self, tag: u8) -> Vec<(BlockId, Marker)> {
        let mut out = Vec::new();
        for (id, b) in self.blocks.iter().enumerate() {
            for m in &b.markers {
                if m.tag == tag {
                    out.push((id, *m));
                }
            }
        }
        out
    }

    fn reachable_preds(&self) -> (Vec<BlockId>, HashMap<BlockId, Vec<BlockId>>) {
        // Reverse postorder DFS from entry.
        let mut visited = HashSet::new();
        let mut postorder = Vec::new();
        let mut stack: Vec<(BlockId, usize)> = vec![(self.entry, 0)];
        visited.insert(self.entry);
        while let Some((node, idx)) = stack.pop() {
            if idx < self.blocks[node].succs.len() {
                let succ = self.blocks[node].succs[idx];
                stack.push((node, idx + 1));
                if visited.insert(succ) {
                    stack.push((succ, 0));
                }
            } else {
                postorder.push(node);
            }
        }
        postorder.reverse();

        let reachable: HashSet<BlockId> = postorder.iter().copied().collect();
        let mut preds: HashMap<BlockId, Vec<BlockId>> = HashMap::new();
        for &n in &postorder {
            for &s in &self.blocks[n].succs {
                if reachable.contains(&s) {
                    preds.entry(s).or_default().push(n);
                }
            }
        }
        (postorder, preds)
    }

    /// Immediate-dominator map for every block reachable from entry (Cooper,
    /// Harvey & Kennedy's iterative algorithm). Unreachable blocks are simply
    /// absent — they cannot dominate anything reachable.
    fn idom(&self) -> HashMap<BlockId, BlockId> {
        let (rpo, preds) = self.reachable_preds();
        let rpo_index: HashMap<BlockId, usize> =
            rpo.iter().enumerate().map(|(i, &b)| (b, i)).collect();

        let mut idom: HashMap<BlockId, BlockId> = HashMap::new();
        idom.insert(self.entry, self.entry);

        let mut changed = true;
        while changed {
            changed = false;
            for &b in rpo.iter() {
                if b == self.entry {
                    continue;
                }
                let mut new_idom: Option<BlockId> = None;
                for &p in preds.get(&b).into_iter().flatten() {
                    if !idom.contains_key(&p) {
                        continue;
                    }
                    new_idom = Some(match new_idom {
                        None => p,
                        Some(cur) => intersect(cur, p, &idom, &rpo_index),
                    });
                }
                if let Some(ni) = new_idom {
                    if idom.get(&b) != Some(&ni) {
                        idom.insert(b, ni);
                        changed = true;
                    }
                }
            }
        }
        idom
    }

    /// Does block `a` dominate block `b`? (`a == b` counts as dominating.)
    /// Both must be reachable from entry; an unreachable block dominates and
    /// is dominated by nothing.
    pub fn block_dominates(&self, a: BlockId, b: BlockId) -> bool {
        let idom = self.idom();
        if !idom.contains_key(&a) || !idom.contains_key(&b) {
            return false;
        }
        let mut cur = b;
        loop {
            if cur == a {
                return true;
            }
            let next = idom[&cur];
            if next == cur {
                return cur == a;
            }
            cur = next;
        }
    }

    /// Does marker `a` dominate marker `b` on every path (with `a` occurring
    /// strictly first when they share a block)?
    pub fn marker_dominates(&self, a_block: BlockId, a: Marker, b_block: BlockId, b: Marker) -> bool {
        if a_block == b_block {
            return a.order < b.order;
        }
        self.block_dominates(a_block, b_block)
    }
}

fn intersect(
    mut a: BlockId,
    mut b: BlockId,
    idom: &HashMap<BlockId, BlockId>,
    rpo_index: &HashMap<BlockId, usize>,
) -> BlockId {
    while a != b {
        while rpo_index[&a] > rpo_index[&b] {
            a = idom[&a];
        }
        while rpo_index[&b] > rpo_index[&a] {
            b = idom[&b];
        }
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use syn::parse_quote;

    fn classify_calls(names: &'static [&'static str]) -> impl Fn(&Expr) -> Option<u8> {
        move |expr: &Expr| match expr {
            Expr::MethodCall(m) => names.iter().position(|n| m.method == n).map(|i| i as u8),
            _ => None,
        }
    }

    #[test]
    fn straight_line_write_dominates_upgrade() {
        let block: Block = parse_quote! {{
            store.write_version();
            deployer.update_current_contract_wasm(h);
        }};
        let empty = HashMap::new();
        let classify = classify_calls(&["write_version", "update_current_contract_wasm"]);
        let cfg = build(&block, &empty, &classify);
        let w = cfg.markers_with_tag(0);
        let u = cfg.markers_with_tag(1);
        assert_eq!(w.len(), 1);
        assert_eq!(u.len(), 1);
        assert!(cfg.marker_dominates(w[0].0, w[0].1, u[0].0, u[0].1));
    }

    #[test]
    fn write_after_upgrade_does_not_dominate() {
        let block: Block = parse_quote! {{
            deployer.update_current_contract_wasm(h);
            store.write_version();
        }};
        let empty = HashMap::new();
        let classify = classify_calls(&["write_version", "update_current_contract_wasm"]);
        let cfg = build(&block, &empty, &classify);
        let w = cfg.markers_with_tag(0);
        let u = cfg.markers_with_tag(1);
        assert!(!cfg.marker_dominates(w[0].0, w[0].1, u[0].0, u[0].1));
    }

    #[test]
    fn write_in_one_if_branch_does_not_dominate_upgrade_after() {
        let block: Block = parse_quote! {{
            if cond {
                store.write_version();
            }
            deployer.update_current_contract_wasm(h);
        }};
        let empty = HashMap::new();
        let classify = classify_calls(&["write_version", "update_current_contract_wasm"]);
        let cfg = build(&block, &empty, &classify);
        let w = cfg.markers_with_tag(0);
        let u = cfg.markers_with_tag(1);
        assert!(!cfg.marker_dominates(w[0].0, w[0].1, u[0].0, u[0].1));
    }

    #[test]
    fn write_before_if_dominates_upgrade_inside_every_branch() {
        let block: Block = parse_quote! {{
            store.write_version();
            if cond {
                deployer.update_current_contract_wasm(h);
            } else {
                deployer.update_current_contract_wasm(h2);
            }
        }};
        let empty = HashMap::new();
        let classify = classify_calls(&["write_version", "update_current_contract_wasm"]);
        let cfg = build(&block, &empty, &classify);
        let w = cfg.markers_with_tag(0);
        let u = cfg.markers_with_tag(1);
        assert_eq!(w.len(), 1);
        assert_eq!(u.len(), 2);
        for (ub, um) in &u {
            assert!(cfg.marker_dominates(w[0].0, w[0].1, *ub, *um));
        }
    }

    #[test]
    fn write_in_while_body_does_not_dominate_code_after_loop() {
        let block: Block = parse_quote! {{
            while cond {
                store.write_version();
            }
            deployer.update_current_contract_wasm(h);
        }};
        let empty = HashMap::new();
        let classify = classify_calls(&["write_version", "update_current_contract_wasm"]);
        let cfg = build(&block, &empty, &classify);
        let w = cfg.markers_with_tag(0);
        let u = cfg.markers_with_tag(1);
        assert!(!cfg.marker_dominates(w[0].0, w[0].1, u[0].0, u[0].1));
    }

    #[test]
    fn write_guarded_by_early_return_still_dominates() {
        let block: Block = parse_quote! {{
            if !authorized {
                return;
            }
            store.write_version();
            deployer.update_current_contract_wasm(h);
        }};
        let empty = HashMap::new();
        let classify = classify_calls(&["write_version", "update_current_contract_wasm"]);
        let cfg = build(&block, &empty, &classify);
        let w = cfg.markers_with_tag(0);
        let u = cfg.markers_with_tag(1);
        assert!(cfg.marker_dominates(w[0].0, w[0].1, u[0].0, u[0].1));
    }

    #[test]
    fn inlines_helper_containing_the_upgrade_call() {
        let caller: Block = parse_quote! {{
            store.write_version();
            self.do_upgrade();
        }};
        let callee: Block = parse_quote! {{
            deployer.update_current_contract_wasm(h);
        }};
        let mut registry = HashMap::new();
        registry.insert("do_upgrade".to_string(), &callee);
        let classify = classify_calls(&["write_version", "update_current_contract_wasm"]);
        let cfg = build(&caller, &registry, &classify);
        let w = cfg.markers_with_tag(0);
        let u = cfg.markers_with_tag(1);
        assert_eq!(w.len(), 1);
        assert_eq!(u.len(), 1);
        assert!(cfg.marker_dominates(w[0].0, w[0].1, u[0].0, u[0].1));
    }

    #[test]
    fn inlined_helper_write_in_branch_does_not_dominate_later_upgrade() {
        let caller: Block = parse_quote! {{
            self.maybe_write();
            deployer.update_current_contract_wasm(h);
        }};
        let callee: Block = parse_quote! {{
            if cond {
                store.write_version();
            }
        }};
        let mut registry = HashMap::new();
        registry.insert("maybe_write".to_string(), &callee);
        let classify = classify_calls(&["write_version", "update_current_contract_wasm"]);
        let cfg = build(&caller, &registry, &classify);
        let w = cfg.markers_with_tag(0);
        let u = cfg.markers_with_tag(1);
        assert!(!cfg.marker_dominates(w[0].0, w[0].1, u[0].0, u[0].1));
    }
}
