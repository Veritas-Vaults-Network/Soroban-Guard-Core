# Checks reference

This document describes what each Soroban Guard Core check looks for and why it matters.

---

## `missing-require-auth` (High)

**Status:** Phase 1

**What it detects**

In an `impl` block marked with `#[contractimpl]` or `#[soroban_sdk::contractimpl]`, any function whose body:

1. Performs a storage mutation through `env.storage()` (heuristic: method calls `set`, `remove`, `extend_ttl`, `bump`, or `append` on a receiver chain that includes `.storage()`), and  
2. Never calls `env.require_auth()` (parameter name **`env`**: `env.require_auth()`).

**Why it matters**

Contract state updates should be gated. This rule only recognizes `env.require_auth()`, not `user.require_auth()` or `env.require_auth_for_args()`.

**Limitations**

- Only the `Env` binding named `env` counts.
- Static analysis cannot see auth hidden in helpers.

**Fixture:** `test-contracts/vulnerable/`, `test-contracts/safe/`

---

## `unchecked-arithmetic` (Medium)

**Status:** Phase 2

**What it detects**

Inside `#[contractimpl]` methods:

- Binary `+`, `-`, `*` where **both** sides are not integer/string literals (so `1 + 2` is ignored, `a + b` is flagged).
- Compound `+=`, `-=`, `*=` (syn 2 represents these as `ExprBinary` with `AddAssign` / `SubAssign` / `MulAssign`).

**Why it matters**

Wrapping arithmetic on `i128` / `u128` amounts can silently overflow. Prefer `checked_*` or `saturating_*` for token math.

**Limitations**

- May flag harmless loop indices; review context.
- Does not analyze types; it is syntactic.

**Fixture:** `test-contracts/arithmetic-vulnerable/`, `test-contracts/arithmetic-safe/`

---

## `unprotected-admin` (High)

**Status:** Phase 2

**What it detects**

Public (`pub fn`) methods in `#[contractimpl]` whose name **exactly matches** a built-in list of sensitive entrypoints (e.g. `set_owner`, `pause`, `migrate`, `upgrade`, … — see `SENSITIVE_NAMES` in `crates/checks/src/admin.rs`), and whose body contains **no** call to `require_auth` or `require_auth_for_args` on any receiver.

**Why it matters**

Names like `set_owner` strongly suggest privilege; without any auth call the scanner treats the entrypoint as world-callable.

**Limitations**

- Name allowlist only; extend the list as your org sees fit.
- Any `require_auth` / `require_auth_for_args` anywhere in the body clears the finding (no dataflow).

**Fixture:** `test-contracts/admin-vulnerable/`, `test-contracts/admin-safe/`

---

## `unsafe-storage-patterns` (Medium)

**Status:** Phase 2

**What it detects**

1. **Temporary storage writes** — `env.storage().temporary()` in the receiver chain of a storage mutation (`set`, `remove`, `extend_ttl`, `bump`, `append`).
2. **Dynamic `Symbol::new` keys** — `Symbol::new(&env, …)` where the second argument is **not** a string literal (e.g. derived from a parameter). Literal second args like `Symbol::new(&env, "fixed")` are ignored.

**Why it matters**

- Temporary data expires with TTL; it is easy to misuse for long-lived balances or ownership.
- Caller-derived symbol strings are easier to enumerate or collide than fixed `symbol_short!` keys.

**Limitations**

- Does not analyze `symbol_short!(...)` macros beyond normal parsing.
- `Symbol::new` with a `const` or macro-expanded literal may still be flagged if it is not a `syn::Lit::Str`.

**Fixture:** `test-contracts/storage-vulnerable/`, `test-contracts/storage-safe/`

---

## `double-init` (High)

**Status:** Phase 2

**What it detects**

Inside `#[contractimpl]` methods whose name contains `"init"` (case-insensitive), this rule flags a storage `set` call through `env.storage()` when the same method does not first perform a storage `has` or `get` guard.

**Why it matters**

Initialization entrypoints usually establish privileged state such as owner/admin or one-time configuration. If they can be called again without checking existing initialization state, an attacker may re-initialize the contract and overwrite critical storage.

**Limitations**

- Structural, not dataflow-aware: the guard must appear as a direct storage `has` or `get` call in the same init-like method.
- Helper-based initialization guards are not recognized.
- Non-init method names are intentionally ignored to avoid flagging normal setters.

**Fixture:** `test-contracts/double-init-vulnerable/`, `test-contracts/double-init-safe/`

---

## `instance-ttl-missing` (Medium)

**Status:** Phase 1

**What it detects**

In a contract file, if there is at least one call to `env.storage().instance().set(...)` but no call to `env.storage().instance().extend_ttl(...)` anywhere in the file.

**Why it matters**

Instance storage in Soroban has a TTL (time-to-live) and will expire if not periodically extended. If a contract uses instance storage but never extends its TTL, the contract may become inaccessible once the instance expires.

**Limitations**

- Only detects direct calls; does not analyze indirect calls through helper functions.
- Checks the entire file, not per function.

**Fixture:** `test-contracts/instance-ttl-vulnerable/`, `test-contracts/instance-ttl-safe/`

---

## `storage-key-collision` (Medium)

**Status:** Phase 1

**What it detects**

Storage keys with similar names that could lead to accidental overwrites, such as "owner", "owner_addr", and "owner_address" in the same contract.

**Why it matters**

Similar key names can cause developers to accidentally use the wrong key when reading or writing storage, leading to data corruption or security vulnerabilities. Distinct key names help prevent these mistakes.

**Limitations**

- Only detects string literal keys, not symbol-based keys
- May flag some legitimate cases where similar keys are intentionally used

**Fixture:** `test-contracts/storage-key-collision-vulnerable/`, `test-contracts/storage-key-collision-safe/`

---

## `balance-not-verified-after-transfer` (Medium)

**Status:** Phase 2

**What it detects**

Inside `#[contractimpl]` methods, any `.transfer(...)` method call on a token client (not on bare `env`) that is not followed by a balance verification.

A balance verification is recognized as:
- A `.balance()` method call on the same token client after the transfer
- An `assert!` or `require!` macro that checks balance values after the transfer

**Why it matters**

External token transfers can fail or behave unexpectedly due to insufficient balance, token contract logic, or other issues. Without verifying the balance after a transfer, the contract may proceed as if the transfer succeeded, leading to accounting errors, incorrect state updates, or potential exploits.

**Limitations**

- The current implementation is conservative and flags all external token transfers without balance verification
- Does not analyze control flow across different branches or functions
- Balance verification must be in the same basic block as the transfer
- Does not distinguish between different token clients in complex scenarios

**Fixture:** `test-contracts/balance-not-verified-after-transfer-vulnerable/`, `test-contracts/balance-not-verified-after-transfer-safe/`

---

## `zero-divisor` (High)

**Status:** Phase 2

**What it detects**

Inside `#[contractimpl]` methods, any `/` (division) or `%` (remainder) where the right-hand operand is a function parameter and the method body does **not** contain a zero-check guard for that parameter anywhere.

A guard is recognized as:

- `assert!(param ...)` — an `assert!` macro whose token stream contains the parameter name (textual heuristic).
- `if cond { ... }` — an `if` expression whose condition contains both the parameter name and the literal `"0"`.

**Why it matters**

Integer division or remainder by zero causes a panic in Rust, which terminates the entire Soroban transaction. An attacker who controls any fee, rate, or price argument can pass `0` to permanently brick any entrypoint that divides by that parameter without checking for zero first.

**Limitations**

- Syntactic, not type-aware: any parameter matching the name triggers the finding; the check does not verify the parameter is actually a numeric type.
- Guards are detected by substring match anywhere in the body, not by dataflow.
- `assert_eq!(param, 0)` (two-argument form) is not recognized — only the single-argument `assert!` form counts.

**Fixture:** `test-contracts/zero-divisor-vulnerable/`, `test-contracts/zero-divisor-safe/`

---

## `timestamp-as-nonce` (High)

**Status:** Phase 2

**What it detects**

Inside `#[contractimpl]` methods, any use of `env.ledger().timestamp()` as a unique nonce or identifier:

1. A `let` binding whose init expression contains an `env.ledger().timestamp()` chain, where the binding's name (lowercased) contains `"nonce"`, `"id"`, or `"unique_id"`.
2. An `env.ledger().timestamp()` chain passed directly as an argument to a storage mutation named `set` (receiver chain containing `.storage()`) — e.g. `env.storage().persistent().set(&env.ledger().timestamp(), &v)`.

Either condition flags the method once.

**Why it matters**

`env.ledger().timestamp()` is the close time of the *current ledger* and is identical for every transaction within that ledger. Using it as a unique nonce, identifier, or storage key means two transactions in the same ledger collide on the same value, enabling replay.

**Limitations**

- Structural/textual, not dataflow: the timestamp chain must appear directly in the flagged binding's init expression or storage-`set` argument, not several variables removed.
- The name heuristic (`"id"` as a substring) can over-match identifiers that merely contain those letters (e.g. `valid`).
- Only the `Env` binding named `env` counts, mirroring `missing-require-auth`.

**Fixture:** `test-contracts/timestamp-as-nonce-vulnerable/`, `test-contracts/timestamp-as-nonce-safe/`

---

## `upgrade-no-schema-version` (Medium)

**Status:** Phase 2

**What it detects**

Across all `#[contractimpl]` methods in a file:

1. Any call to `update_current_contract_wasm` - the containing function is recorded as the upgrade function.
2. If no such call is found, the check produces no findings.
3. Separately, any call to `set` (on a receiver chain containing `.storage()`) where the first argument's token string (lowercased) contains `"version"` or `"schema"` - searched across all functions in the file.
4. If step 3 finds nothing, the upgrade function from step 1 is flagged.

**Why it matters**

When a contract upgrades itself via `env.deployer().update_current_contract_wasm(...)`, the storage layout may change between versions. Without a schema or version key in persistent storage, the new code has no way to detect it is reading data written by an older layout. This leads to silent corruption or panics on deserialization.

**Limitations**

- Syntactic, not semantic: any storage `set` call whose key tokens contain `"version"` or `"schema"` satisfies the check regardless of storage tier or actual runtime value.
- Does not verify the version key is written atomically with the upgrade call, only that it exists somewhere in the file.
- Token matching is case-insensitive but purely textual - a key named `SCHEMA_VERSION` constant satisfies it only if those characters appear in the token stream at the call site.

**Fixture:** `test-contracts/upgrade-no-schema-version-vulnerable/`, `test-contracts/upgrade-no-schema-version-safe/`

---

## `interprocedural-supply-cap-bypass` (High)

**Status:** Phase 3

**What it detects**

`mint-no-cap` and `supply-cap-not-enforced` (see above) each look at a single function named `mint` in isolation. That misses a whole-file gap: a second way to increase total supply — an admin, emergency, or migration entrypoint, or a helper reachable from more than one entrypoint — that increments the same total-supply key but never repeats the cap check that `mint` enforces on its own path.

This check builds a small, file-local, name-resolved call graph and reasons about it per entrypoint:

1. **Registry** — every named function body in the file is collected: `#[contractimpl]` methods, methods in any other `impl` block (a common place to put private helpers), and free `fn` items.
2. **Call edges** — inside each function body, every `syn::ExprCall` whose callee is a path expression (`foo(..)`, `Self::foo(..)`, `Type::foo(..)`) contributes an edge to the last path segment's identifier, if that name exists in the registry.
3. **Per-function summary** — each function body is scanned (matching the same heuristics as `mint-no-cap`) for: a storage `set` call on a receiver chain containing `.storage()` whose **key argument** contains a supply-key hint (`supply`, `total`, `minted`, `cap`, `max`, case-insensitive), and any `<=`/`<` binary comparison anywhere in the body (a proxy for a cap check, including inside `assert!`/`assert_eq!` macro bodies).
4. **Reachability** — for every `pub fn` inside a `#[contractimpl]` block, the check does a breadth-first walk of the call graph starting at that entrypoint to compute its full reachable set of functions (itself plus every helper it can reach, transitively).
5. **Per-entrypoint verdict** — within *that entrypoint's own reachable set* (not the whole file), if any reachable function writes the total-supply key and **no** reachable function contains a cap comparison, the entrypoint is flagged. This check is intentionally repeated independently per entrypoint: a cap check reachable from `mint` does **not** clear a finding on `emergency_mint` unless `emergency_mint` can also reach it (e.g. by calling the same checked helper).

**Why it matters**

A single-function check like `mint-no-cap` will pass a file where `mint` is fully capped but a second entrypoint (`emergency_mint`, `admin_mint`, `migrate_supply`, …) writes the identical total-supply storage value with no cap comparison anywhere on its call path. The cap is enforced only on paper, and the second entrypoint silently allows unbounded inflation. Because Soroban contracts commonly factor shared logic into private helpers, a bypass can also hide one call away rather than inline in the entrypoint body.

**Limitations**

- **Name-resolved, not type-resolved**: call edges are matched on the callee's final path segment identifier against the file's function registry. Two differently-scoped functions that happen to share a name will be treated as the same node; a call through a trait object, closure, or function pointer is invisible to this graph.
- **File-local only**: the registry and call graph are built from a single `syn::File`. A helper defined in another module or crate is not resolved, so a supply write that only happens through a cross-file call will not be seen by this check (consistent with every other check in this analyzer, which runs per-file with no shared state).
- **Same key heuristic as `mint-no-cap`/`supply-cap-not-enforced`**: the total-supply key is recognized by a hint substring on the `set` call's key argument, not by tracking a specific storage key value across functions. A `set` call whose key argument happens to contain one of the hint words but is unrelated to total supply can still be treated as a supply write.
- **`<=`/`<` anywhere in the reachable set, not tied to the write**: as with the single-function checks, any comparison using `Le`/`Lt` counts as a cap check, even if it does not actually bound the specific value being written.
- Does not model recursion depth or short-circuiting; the BFS simply visits each reachable function once.

**Fixture:** `test-contracts/interprocedural-supply-cap-vulnerable/`, `test-contracts/interprocedural-supply-cap-safe/`
