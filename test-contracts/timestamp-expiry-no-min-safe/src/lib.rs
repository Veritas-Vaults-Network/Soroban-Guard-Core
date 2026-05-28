#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

const MIN_DURATION: u64 = 3600;

#[contract]
pub struct TimestampExpiryNoMinSafe;

#[contractimpl]
impl TimestampExpiryNoMinSafe {
    /// Adds minimum duration to timestamp before storing — safe.
    pub fn set_expiry_with_min(env: Env) {
        let expiry = env.ledger().timestamp() + MIN_DURATION;
        env.storage().instance().set(&"expiry", &expiry);
    }

    /// Validates duration parameter before storing timestamp — safe.
    pub fn set_expiry_with_guard(env: Env, duration: u64) {
        if duration >= MIN_DURATION {
            let expiry = env.ledger().timestamp();
            env.storage().instance().set(&"expiry", &expiry);
        }
    }
}
