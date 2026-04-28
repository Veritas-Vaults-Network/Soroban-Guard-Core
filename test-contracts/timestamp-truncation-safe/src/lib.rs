#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct TimeLockSafe;

#[contractimpl]
impl TimeLockSafe {
    /// ✅ `unlock_time` is `u64` — full timestamp range, no truncation.
    pub fn lock(env: Env, unlock_time: u64) {
        assert!(env.ledger().timestamp() >= unlock_time);
    }
}
