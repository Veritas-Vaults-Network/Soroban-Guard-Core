#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct NegativeDepositSafe;

/// Safe: asserts amount > 0 before writing to storage.
#[contractimpl]
impl NegativeDepositSafe {
    pub fn deposit(env: Env, from: Address, amount: i128) {
        from.require_auth();
        // Safe: negative amounts are rejected before any state change.
        assert!(amount > 0, "deposit amount must be positive");
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
