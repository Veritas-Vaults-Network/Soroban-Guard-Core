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

## `oracle-price-staleness` (High)

**Status:** Phase 3

**What it detects**

Unlike every other check in this crate — a single `syn::visit::Visit` walk over one
function body — this check builds a small **call graph** over every function defined in
the file (free functions and every method in every `impl` block, not just
`#[contractimpl]` ones) and asks a reachability question instead of a syntactic one.

1. **Find the tracked price.** Per function, look for a `let` binding whose initializer
   is a struct literal with a field whose expression contains an
   `env.ledger().timestamp()` chain (the "freshness field", e.g. `last_updated`) and at
   least one other field (the "price fields", e.g. `price`). Then find where that local
   (or an inline struct literal) is passed to `<...storage...>.set(&KEY, &value)` and
   record `KEY` together with the freshness field name and price field names. If no such
   write pattern exists anywhere in the file, the check produces no findings.
2. **Find every read site.** Any function containing `<...storage...>.get(&KEY)` for a
   tracked `KEY` is a read site.
3. **Find every freshness-check function.** Any function containing a comparison
   (`<`, `<=`, `>`, `>=`) where one side's subtree contains a timestamp signal
   (`env.ledger().timestamp()` inline, or a local bound to it, e.g.
   `let now = env.ledger().timestamp();`) and the other side's subtree contains the
   tracked freshness field (by name) is a freshness checker. `assert!`/`panic!` macro
   bodies are parsed as a comma-separated expression list so `assert!(cond, "msg")`
   still exposes `cond`.
4. **Find every arithmetic-use site.** Any function that performs `+ - * / %` on a value
   sourced from the tracked key — a local bound from the `.get(&KEY)` call, a `.field`
   access on that local matching a tracked price field name, or the `.get(&KEY)` chain
   used inline — is a use site.
5. **Reachability.** Build a directed call graph (caller → callee edges resolved by
   matching call/method-call idents against known function names in the file — no type
   inference, same heuristic style as the rest of this crate). For every public
   `#[contractimpl]` entry point, compute its forward-reachable set (every function it
   transitively calls, including itself). For each read site `R`, take the union of the
   reachable sets of every entry point whose reachable set contains `R` (or, if none
   does, `R`'s own reachable set as a fallback root). If that union contains an
   arithmetic-use site but **no** freshness-check function anywhere in it, flag the
   arithmetic-use site.

This is why a `check_price_fresh` helper clears the finding even when it is called from a
*different* function than the one that ultimately does the arithmetic: the helper only
has to be reachable from the same public entry point as the read and the use, not textually
inside the same function body.

**Why it matters**

A price paired with a `last_updated` timestamp is only as safe as the *weakest* function
that reads it. If even one code path fetches the price and uses it in a calculation
without checking `env.ledger().timestamp() - last_updated <= MAX_AGE` (or similar)
anywhere on its call path, a stalled or manipulated oracle feed can be used for pricing
indefinitely.

**Limitations**

- **Order-insensitive.** Reachability does not model control flow or call order: a
  freshness check that runs *after* the price is already used (or on a branch that never
  actually executes before the use) still clears the finding, because there is no CFG or
  dominance analysis here — only "is this function present in the transitively-called
  set." A real interprocedural analysis would need a CFG to enforce ordering.
  Contributions welcome.
- **Name-based call resolution.** Calls are matched by identifier name against functions
  defined in the same file, not by type-checked method resolution. Two identically named
  methods on different types in the same file, or a call resolved through a trait object,
  can produce a spurious edge or miss one entirely.
- **File-scoped.** Like every check in this crate, the call graph is built from a single
  parsed `syn::File`; a freshness check defined in a different file (module) is invisible.
- **Struct-literal pattern only.** The write-site pattern requires a struct literal with a
  timestamp-bearing field (inline in the `.set()` call, or bound to a local first). A
  price and its timestamp stored as two independent `.set()` calls to unrelated keys is
  not correlated and will not be tracked.
- **No taint through return values.** If a helper calls `.get(&KEY)` and *returns* the
  price to its caller (rather than the caller calling `.get()` directly), the caller's use
  of the returned value is not traced back to the tracked key — only direct reads and
  direct field access on a locally-bound `.get()` result are tracked.

**Fixture:** `test-contracts/oracle-price-staleness-vulnerable/`, `test-contracts/oracle-price-staleness-safe/`
