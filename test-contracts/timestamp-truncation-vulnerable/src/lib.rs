#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct TimeLockVulnerable;

#[contractimpl]
impl TimeLockVulnerable {
    /// ❌ `unlock_time` is `u32` — truncates after year 2106.
    pub fn lock(env: Env, unlock_time: u32) {
        assert!(env.ledger().timestamp() >= unlock_time as u64);
    }
}
