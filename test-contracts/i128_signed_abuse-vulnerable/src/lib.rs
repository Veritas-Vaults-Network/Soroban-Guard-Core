#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct I128SignedAbuseVulnerable;

#[contractimpl]
impl I128SignedAbuseVulnerable {
    // ❌ balance, supply, and count are semantically non-negative but typed i128,
    //    allowing callers to pass negative values without a type-level guard.
    pub fn mint(env: Env, balance: i128, supply: i128) {
        let _ = (env, balance, supply);
    }

    pub fn set_limit(env: Env, cap: i128, limit: i128) {
        let _ = (env, cap, limit);
    }

    pub fn record(env: Env, count: i128, total: i128, amount: i128) {
        let _ = (env, count, total, amount);
    }
}
