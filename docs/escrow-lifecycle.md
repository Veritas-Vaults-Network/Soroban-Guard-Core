# Escrow Lifecycle

This document describes the complete state machine for the LiquiFact escrow contract, including the `Cancelled` state and investor refund path added in [#242](https://github.com/Liquifact/Liquifact-contracts/issues/242).

---

## Status codes

| Code | Name        | Description                                                  |
|------|-------------|--------------------------------------------------------------|
| `0`  | Funding     | Accepting investor contributions; target not yet reached.    |
| `1`  | Active      | Funding target met; escrow obligations are live.             |
| `2`  | Completed   | All escrow obligations fulfilled.                            |
| `3`  | Legal Hold  | Frozen by a compliance action; most transitions blocked.     |
| `4`  | Cancelled   | Funding failed; investors may claim refunds.                 |

---

## State machine

```
                    ┌─────────────────────────────────────────────────┐
                    │                                                 │
  initialize()      ▼                                                 │
  ──────────►  0 (Funding) ──── admin cancel_funding() ──────► 4 (Cancelled)
                    │                                                 │
                    │  (target reached, off-chain trigger)            │  investor refund()
                    ▼                                                 ▼
               1 (Active)                                  InvestorContribution → 0
                    │
                    │  (obligations fulfilled)
                    ▼
               2 (Completed)
```

Legal hold (`status = 3`) can be set from any non-terminal state by a compliance action and blocks `cancel_funding`.

---

## Entrypoints

### `initialize(admin, funding_token, funding_target)`

Sets up the escrow. Can only be called once. Sets status to `0`.

- `admin` — address authorised to call `cancel_funding`.
- `funding_token` — SEP-41 token contract address used for contributions and refunds.
- `funding_target` — minimum total contribution (in stroops) to transition to Active.

### `contribute(investor, amount)`

Records an investor contribution during the Funding phase.

- Requires `investor` authorisation.
- Transfers `amount` from `investor` to the contract.
- Accumulates `DataKey::InvestorContribution(investor)` and `DataKey::FundedAmount`.
- Panics if status ≠ `0` or `amount ≤ 0`.

### `cancel_funding()`

Transitions status from `0` (Funding) to `4` (Cancelled).

- Requires `admin` authorisation.
- Panics if status ≠ `0` or legal hold is active.
- Emits `FundingCancelled { admin, funded_amount }`.

### `refund(investor)`

Returns an investor's contribution after cancellation.

- Requires `investor` authorisation.
- Panics if status ≠ `4` or `InvestorContribution(investor) == 0`.
- Zeroes `InvestorContribution(investor)` **before** the token transfer (prevents double-spend).
- Transfers exactly the recorded contribution via `external_calls::transfer_funding_token_with_balance_checks`.
- Emits `Refunded { investor, amount }`.

---

## Events

### `FundingCancelled`

Emitted by `cancel_funding`.

| Field           | Type      | Description                              |
|-----------------|-----------|------------------------------------------|
| `admin`         | `Address` | Admin that triggered the cancellation.   |
| `funded_amount` | `i128`    | Total contributions at time of cancel.   |

### `Refunded`

Emitted by `refund`.

| Field      | Type      | Description                              |
|------------|-----------|------------------------------------------|
| `investor` | `Address` | Investor who received the refund.        |
| `amount`   | `i128`    | Amount returned (in stroops).            |

---

## Storage layout

| Key                              | Tier       | Type      | Description                          |
|----------------------------------|------------|-----------|--------------------------------------|
| `DataKey::Admin`                 | Instance   | `Address` | Contract administrator.              |
| `DataKey::FundingToken`          | Instance   | `Address` | SEP-41 token address.                |
| `DataKey::FundingTarget`         | Instance   | `i128`    | Minimum funding goal.                |
| `DataKey::FundedAmount`          | Instance   | `i128`    | Running total of contributions.      |
| `DataKey::Status`                | Instance   | `u32`     | Current lifecycle status code.       |
| `DataKey::LegalHold`             | Instance   | `bool`    | Compliance freeze flag.              |
| `DataKey::InvestorContribution(addr)` | Persistent | `i128` | Per-investor contribution amount. |

Instance storage is extended on `initialize` and should be bumped by callers on active use. Persistent `InvestorContribution` entries are extended on each `contribute` call.

---

## Security invariants

| Invariant | Enforcement |
|-----------|-------------|
| Only admin can cancel | `admin.require_auth()` at top of `cancel_funding` |
| Only investor can refund | `investor.require_auth()` at top of `refund` |
| No double-refund | Contribution zeroed before transfer; second call panics on `amount == 0` check |
| No refund outside Cancelled state | Status assertion at top of `refund` |
| No cancel outside Funding state | Status assertion at top of `cancel_funding` |
| Legal hold blocks cancel | `legal_hold` assertion in `cancel_funding` |
| Balance-delta check on every transfer | `transfer_funding_token_with_balance_checks` asserts `balance_after - balance_before == amount` |
| Overflow-safe arithmetic | `checked_add` / `checked_sub` throughout; `overflow-checks = true` in release profile |
| Total refunded ≤ funded_amount | Each refund ≤ individual contribution; sum of contributions == `funded_amount` |

---

## Example flow

```
# Funding phase
initialize(admin, USDC, 1_000_000)
contribute(alice, 600_000)   # alice's contribution recorded
contribute(bob,   400_000)   # funded_amount == 1_000_000

# Admin decides to cancel (target not reached in time, or other reason)
cancel_funding()             # status → 4, FundingCancelled emitted

# Investors reclaim principal
refund(alice)                # 600_000 USDC returned, Refunded emitted
refund(bob)                  # 400_000 USDC returned, Refunded emitted

# Second refund attempt is rejected
refund(alice)                # panics: "no contribution to refund"
```
