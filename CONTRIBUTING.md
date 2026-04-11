# Contributing to Soroban Guard Core

Thank you for helping improve the static analyzer. This guide covers **local setup**, a **short `syn` tutorial with examples**, **how to add a check** (using `auth.rs` as a template), **how to write test contracts**, and links to **sister repositories** in the Veritas Vaults ecosystem.

## Local development setup

1. **Install Rust** (1.74 or newer recommended) using [rustup](https://rustup.rs/):

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source "$HOME/.cargo/env"
   rustc --version
   ```

2. **Clone this repository** and `cd` into the workspace root:

   ```bash
   git clone https://github.com/Veritas-Vaults-Network/soroban-guard-core.git
   cd soroban-guard-core
   ```

3. **Build and run the full test suite:**

   ```bash
   cargo build --workspace
   cargo test --workspace
   ```

4. **Run the CLI** against bundled fixtures:

   ```bash
   cargo run -p soroban-guard-cli -- scan test-contracts/vulnerable
   cargo run -p soroban-guard-cli -- scan test-contracts/safe --json
   ```

5. **Standalone Soroban crates** — Paths under `test-contracts/` are listed in `[workspace.exclude]`. To type-check a fixture on its own:

   ```bash
   cd test-contracts/arithmetic-safe && cargo check
   ```

6. **Install the `soroban-guard` binary** (optional):

   ```bash
   cargo install --path crates/cli
   ```

### Commit hygiene

Prefer **small, focused commits** (one logical change per commit): a single check, a doc section, or a test fixture pair. This makes review and `git bisect` straightforward. Aim for **clear commit messages** in [Conventional Commits](https://www.conventionalcommits.org/) style (`feat(checks): …`, `fix(cli): …`, `docs: …`).

---

## Mini tutorial: `syn` and the AST (with code examples)

The workspace enables **`syn` with the `full` feature** (see root `Cargo.toml` → `[workspace.dependencies]`) so every `Item`, `Expr`, and `Stmt` variant is available for pattern matching and visitors. **`proc-macro2`** is configured with **`span-locations`** so `expr.span().start().line` maps to a 1-based source line when parsing whole files.

### Walk the crate root

`syn::parse_file` returns a `syn::File`. Its `items` slice holds top-level declarations (`use`, `struct`, `impl`, …):

```rust
use syn::{parse_file, Item};

fn list_struct_names(src: &str) -> Result<Vec<String>, syn::Error> {
    let file = parse_file(src)?;
    let mut names = Vec::new();
    for item in &file.items {
        if let Item::Struct(s) = item {
            names.push(s.ident.to_string());
        }
    }
    Ok(names)
}
```

### Visit expressions without listing every `Expr` variant

Implementing [`syn::visit::Visit`](https://docs.rs/syn/latest/syn/visit/trait.Visit.html) dispatches recursively. This mirrors how `auth.rs` and `overflow.rs` detect method calls and binary operators:

```rust
use syn::visit::{self, Visit};
use syn::{ExprBinary, ExprMethodCall, BinOp};

#[derive(Default)]
struct StorageCallCount {
    storage_methods: usize,
    unchecked_int_ops: usize,
}

impl<'ast> Visit<'ast> for StorageCallCount {
    fn visit_expr_method_call(&mut self, i: &'ast ExprMethodCall) {
        if i.method == "storage" {
            self.storage_methods += 1;
        }
        visit::visit_expr_method_call(self, i);
    }

    fn visit_expr_binary(&mut self, i: &'ast ExprBinary) {
        if matches!(i.op, BinOp::Add(_) | BinOp::Sub(_) | BinOp::Mul(_)) {
            self.unchecked_int_ops += 1;
        }
        visit::visit_expr_binary(self, i);
    }
