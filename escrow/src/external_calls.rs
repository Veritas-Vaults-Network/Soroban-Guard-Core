//! External contract call helpers.
//!
//! All token transfers go through this module so that balance-delta checks are
//! applied consistently and cannot be bypassed by callers.

use soroban_sdk::{token, Address, Env};

/// Transfer `amount` of `token` from `from` to `to`, asserting that the
/// recipient's balance increases by exactly `amount` (balance-delta check).
///
/// # Panics
/// - If `amount` is not positive.
/// - If the recipient's post-transfer balance does not equal `pre + amount`
///   (guards against fee-on-transfer tokens or re-entrancy that drains the
///   contract mid-call).
///
/// # Security notes
/// - The balance snapshot is taken **before** the transfer call.
/// - The assertion is evaluated **after** the transfer returns.
/// - Because Soroban's execution model is single-threaded and re-entrant calls
///   within the same transaction are serialised, this check is sufficient to
///   detect unexpected balance changes caused by malicious token contracts.
pub fn transfer_funding_token_with_balance_checks(
    env: &Env,
    token: &Address,
    from: &Address,
    to: &Address,
    amount: i128,
) {
    assert!(amount > 0, "transfer amount must be positive");

    let client = token::Client::new(env, token);

    let balance_before: i128 = client.balance(to);

    client.transfer(from, to, &amount);

    let balance_after: i128 = client.balance(to);

    // Checked arithmetic: balance_after - balance_before must equal amount.
    let delta = balance_after
        .checked_sub(balance_before)
        .expect("balance underflow after transfer");

    assert_eq!(
        delta, amount,
        "balance delta mismatch: expected {amount}, got {delta}"
    );
}
