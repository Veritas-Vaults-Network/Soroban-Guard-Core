#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct ExpiredDeadlineVulnerable;

/// Vulnerable: stores expiry without checking it against the current ledger timestamp.
/// A caller can set an already-expired deadline, bypassing time-lock logic.
#[contractimpl]
impl ExpiredDeadlineVulnerable {
    pub fn set_expiry(env: Env, expiry: u64) {
        // BUG: no check that expiry > env.ledger().timestamp()
        env.storage()
            .persistent()
            .set(&symbol_short!("expiry"), &expiry);
    }
}
