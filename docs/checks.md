# Checks reference

This document describes what each Soroban Guard Core check looks for and why it matters.

---

## `missing-require-auth` (High)

**Status:** implemented (Phase 1)

**What it detects**

In an `impl` block marked with `#[contractimpl]` or `#[soroban_sdk::contractimpl]`, any function whose body:

1. Performs a storage mutation through `env.storage()` (heuristic: method calls `set`, `remove`, `extend_ttl`, `bump`, or `append` on a receiver chain that includes `.storage()`), and  
2. Never calls `env.require_auth()` (literal parameter name **`env`**: `env.require_auth()`).

**Why it matters**

Contract state updates should be gated so only authorized accounts invoke them. Missing `env.require_auth()` on the Soroban `Env` means the scanner cannot see an explicit env-level auth check before writes (note: `user.require_auth()` on an `Address` is **not** treated as `env.require_auth()` for this rule).

**Limitations**

- Only recognizes the `Env` binding named `env`. If you rename it (e.g. `e.require_auth()`), the check may false-positive.
- `env.require_auth_for_args(...)` is **not** counted as satisfying this rule (only `env.require_auth()`).
- Static analysis cannot prove auth happens indirectly through helpers without inlining heuristics.

---

## `unchecked-arithmetic` (Medium)

**Status:** placeholder (no findings yet)

Reserved for unchecked wrapping arithmetic (`+`, `-`, `*`, etc.) on Soroban token amounts and similar.

---

## `unprotected-admin` (Medium)

**Status:** placeholder (no findings yet)

Reserved for privileged operations (pause, upgrade, role changes) without access control patterns.

---

## `unsafe-storage-patterns` (Low)

**Status:** placeholder (no findings yet)

Reserved for risky key design, mixing instance vs persistent storage, or other storage footguns.
