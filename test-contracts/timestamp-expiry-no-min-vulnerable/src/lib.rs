#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct TimestampExpiryNoMinVulnerable;

#[contractimpl]
impl TimestampExpiryNoMinVulnerable {
    /// Stores timestamp directly as expiry without minimum duration guard.
    /// Should trigger `timestamp-expiry-no-min` (Medium).
    pub fn set_expiry(env: Env) {
        env.storage()
            .instance()
            .set(&"expiry", &env.ledger().timestamp());
    }

    /// Stores timestamp inline without guard.
    /// Should trigger `timestamp-expiry-no-min` (Medium).
    pub fn set_deadline(env: Env) {
        let deadline = env.ledger().timestamp();
        env.storage().instance().set(&"deadline", &deadline);
    }
}
