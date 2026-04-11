# Contributing to Soroban Guard Core

## Adding a new check

1. **Pick a crate module** under `crates/checks/src/` (or add `my_check.rs` and `pub mod my_check` in `lib.rs`).

2. **Implement `Check`** from `soroban-guard-checks`:

```rust
use soroban_guard_checks::{Check, Finding, Severity};
use syn::File;

pub struct MyCheck;

impl Check for MyCheck {
    fn name(&self) -> &str {
        "my-check-id"
    }

    fn run(&self, file: &File, source: &str) -> Vec<Finding> {
        let mut out = Vec::new();
        // Inspect `file` (AST). Use `source` for byte offsets / line slicing if needed.
        out
    }
}
```

3. **Register** your type in `default_checks()` in `crates/checks/src/lib.rs`.

4. **Document** the rule in `docs/checks.md`.

5. **Test** with unit tests inside the module (`syn::parse_file` on sample Rust) and optional fixtures under `test-contracts/`.

**Note:** The analyzer sets `Finding::file_path` after `run` returns; leave `file_path` empty in the `Finding` you construct.

---

## Mini tutorial: `syn` for this codebase

- **`syn::parse_file(&str)`** — parse a whole Rust file into `syn::File` (`items` are top-level `Item` enums).

- **Walk items** — match on `Item::Fn`, `Item::Impl`, etc. For Soroban, `Item::Impl` carries `attrs` (e.g. `contractimpl`) and `items` (methods).

- **Visit expressions** — `syn::visit::Visit` lets you recurse without hand-writing every `Expr` variant. Implement `visit_expr_method_call` for call patterns (e.g. `.storage().persistent().set(...)`).

- **Spans and lines** — enable `proc-macro2`’s `span-locations` feature (already in this workspace) so `expr.span().start().line` is meaningful when parsing source files.

- **Attributes** — `attr.path()` returns the path (`contractimpl` vs `soroban_sdk::contractimpl`).

For full API details, see the [syn documentation](https://docs.rs/syn/).

---

## Code style

- Prefer small, testable functions over giant visitors.
- Keep CLI output stable for scripting when using `--json`.
