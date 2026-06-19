#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct BalanceMulOverflowSafe;

/// Safe: uses checked_mul to prevent silent overflow on large balances.
#[contractimpl]
impl BalanceMulOverflowSafe {
    pub fn apply_interest(env: Env, rate: i128) {
        let balance: i128 = env
            .storage()
            .persistent()
            .get(&symbol_short!("bal"))
            .unwrap_or(0);
        // Safe: checked_mul panics on overflow instead of silently wrapping.
        let new_balance = balance.checked_mul(rate).expect("overflow");
        env.storage()
            .persistent()
            .set(&symbol_short!("bal"), &new_balance);
    }
}
