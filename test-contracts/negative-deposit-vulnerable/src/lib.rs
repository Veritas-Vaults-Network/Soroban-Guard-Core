#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct NegativeDepositVulnerable;

/// Vulnerable: accepts a negative i128 amount without a positive-value guard.
/// A caller passing a negative amount effectively withdraws while appearing to deposit.
#[contractimpl]
impl NegativeDepositVulnerable {
    pub fn deposit(env: Env, from: Address, amount: i128) {
        from.require_auth();
        // BUG: no check that amount > 0.
        let balance: i128 = env
            .storage()
            .persistent()
            .get(&from)
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&from, &(balance + amount));
    }
}
