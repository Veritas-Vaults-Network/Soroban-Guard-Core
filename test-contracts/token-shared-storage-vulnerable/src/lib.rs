#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct TokenSharedStorageVulnerable;

#[contractimpl]
impl TokenSharedStorageVulnerable {
    /// ❌ Token keys and governance keys share the same storage namespace.
    /// A key collision between "balance_user" and "stake_amount" could corrupt data.
    pub fn deposit(env: Env, amount: i128) {
        env.storage().instance().set(&"balance_user", &amount);
    }

    pub fn vote(env: Env, proposal: u32) {
        env.storage().instance().set(&"proposal_id", &proposal);
    }

    pub fn stake(env: Env, amount: i128) {
        env.storage().instance().set(&"stake_amount", &amount);
    }
}
