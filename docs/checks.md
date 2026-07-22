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

## `cross-token-provenance-mix` (High)

**Status:** Phase 3

**What it detects**

This check is different in kind from every other rule in this document: it is the only
one that performs def-use/taint-style tracing of a value's *provenance* across an
arithmetic expression rather than a single-expression syntactic pattern match. It
targets a bug class specific to functions that handle two (or more) assets: combining
amounts that are denominated in different tokens with `+`/`-` as though they were the
same unit — e.g. `let total = amount_a + amount_b;` in a two-token `swap` — which is
almost never correct unless an explicit conversion happened first.

The algorithm, in `#[contractimpl]` methods:

1. **Identify the "asset" parameters.** Collect every `Address`-typed parameter whose
   name (lowercased) contains `token` or `asset` (e.g. `token_a`, `token_b`). If fewer
   than two such parameters exist, the function is not a candidate for this check at
   all and is skipped entirely — this is the gate that keeps the check from firing on
   ordinary single-token methods that merely happen to take more than one `Address`
   (caller, recipient, admin, ...).
2. **Tag numeric parameters.** For every `i128`/`u128`/`i64`/`u64`/`i32`/`u32`
   parameter, pair it with one of the asset parameters by naming convention: first,
   match the trailing `_<suffix>` of both names (`amount_a` <-> `token_a` via the
   shared suffix `a`); failing that, check whether the numeric parameter's name
   textually contains an asset parameter's full name (`token_a_amount` contains
   `token_a`). A numeric parameter that matches neither rule is left untagged and
   invisible to the rest of the check.
3. **Propagate tags.** Walking the function body in source order, each tag is carried
   forward through `let` rebindings (`let a = amount_a;` tags `a` the same as
   `amount_a`) and through `+`/`-` arithmetic where both operands carry the *same* tag
   (the result keeps that tag). A `let` whose initializer resolves to no tag (or to a
   mismatch — see next point) drops any previous tag for that binding name, so a
   shadowed name doesn't keep stale provenance.
4. **Flag mismatches.** Any `+`, `-`, `+=`, or `-=` expression whose two operands
   resolve to two *different* asset tags is flagged, **unless** a call whose name
   contains `rate`, `price`, `convert`, or `exchange` (case-insensitive; `ExprCall` or
   `ExprMethodCall`) was encountered earlier in the same source-order traversal of the
   function body. That call is treated as an explicit unit conversion and suppresses
   every mismatch found afterward in that function — this is a whole-function
   suppression, not a check that the conversion call's result is actually the operand
   being combined.

`*`/`/` are deliberately **not** treated as "combining" two denominations: multiplying
or dividing values from two different assets is a normal way to compute an exchange
rate or a price (`amount_a * price_b`), whereas adding or subtracting them essentially
never is.

**Why it matters**

Two amounts from two different tokens are not the same unit of account. Summing or
differencing them directly - without first converting one side through an oracle price
or fixed exchange rate - produces a number that has no real-world meaning, and using
that number to move funds, record a balance, or check a threshold can under- or
over-account one side of a swap, letting an attacker drain value or corrupt accounting
state.

**Limitations (fuzzy by construction, since `syn` has no type information)**

- **Naming-convention dependent, in both directions.** The whole check hinges on the
  `token_a`/`amount_a`-style suffix or substring convention; a swap that names its
  parameters `sell_token`/`buy_token` and `amount_in`/`amount_out` (no shared suffix
  and no substring containment) will simply not be tagged, and the check produces
  **no findings** for genuinely mixed arithmetic — a false negative by design rather
  than a guess.
- **No real def-use graph.** Propagation is a single forward pass over the `syn` AST in
  traversal order (`syn::visit::Visit`'s default depth-first walk), not a true
  control-flow graph: a tag established only inside one branch of an `if`/`else` is
  still visible to code that (in a real CFG) could only run when the *other* branch
  was taken, and loops are not modeled as repeating.
- **Whole-function conversion suppression.** A conversion-looking call anywhere earlier
  in the function suppresses **every** later mismatch in that function, whether or not
  its return value is the one actually used in the flagged expression. A `convert_rate`
  call unrelated to the amounts being combined would incorrectly silence a real finding.
- **Two-asset case only in practice.** With three or more asset parameters, any pair of
  differently-tagged operands is still flagged, but the tag propagation and
  conversion-suppression heuristics above were designed and tested against the
  two-asset (`token_a`/`token_b`) case.
- **Method calls are not modeled as arithmetic.** `amount_a.checked_add(amount_b)` is
  not traced — only literal `+`/`-`/`+=`/`-=` binary expressions are.

**Fixture:** `test-contracts/cross-token-provenance-mix-vulnerable/`, `test-contracts/cross-token-provenance-mix-safe/`
