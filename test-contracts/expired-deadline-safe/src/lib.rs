#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct ExpiredDeadlineSafe;

/// Safe: validates expiry against the current ledger timestamp before storing.
#[contractimpl]
impl ExpiredDeadlineSafe {
    pub fn set_expiry(env: Env, expiry: u64) {
        // Safe: reject already-expired deadlines.
        assert!(expiry > env.ledger().timestamp(), "expiry must be in the future");
        env.storage()
            .persistent()
            .set(&symbol_short!("expiry"), &expiry);
    }
}
