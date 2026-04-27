#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct BalanceMulOverflowVulnerable;

/// Vulnerable: multiplies a storage balance without checked_mul.
/// On large balances this silently overflows, producing incorrect results.
#[contractimpl]
impl BalanceMulOverflowVulnerable {
    pub fn apply_interest(env: Env, rate: i128) {
        // BUG: unchecked multiplication — can overflow on large balances.
        let balance: i128 = env
            .storage()
            .persistent()
            .get(&symbol_short!("bal"))
            .unwrap_or(0);
        let new_balance = balance * rate;
        env.storage()
            .persistent()
            .set(&symbol_short!("bal"), &new_balance);
    }
}
