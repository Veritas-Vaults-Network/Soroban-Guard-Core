#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct TokenSharedStorageSafe;

#[contractimpl]
impl TokenSharedStorageSafe {
    /// ✅ Only token-domain keys — no governance/staking keys in the same namespace.
    pub fn deposit(env: Env, amount: i128) {
        env.storage().instance().set(&"balance_user", &amount);
    }

    pub fn approve(env: Env, amount: i128) {
        env.storage().instance().set(&"allowance_key", &amount);
    }
}
