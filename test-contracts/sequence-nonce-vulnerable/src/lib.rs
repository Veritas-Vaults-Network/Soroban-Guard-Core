#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct SequenceNonceVulnerable;

#[contractimpl]
impl SequenceNonceVulnerable {
    /// Uses env.ledger().sequence() in comparison without storing it — vulnerable.
    pub fn vulnerable_nonce_check(env: Env, nonce: u32) {
        if env.ledger().sequence() == nonce {
            // Do something without storing the nonce
        }
    }

    /// Another vulnerable pattern with > comparison.
    pub fn vulnerable_greater_check(env: Env, nonce: u32) {
        if env.ledger().sequence() > nonce {
            // Process without persistence
        }
    }

    /// Uses sequence in >= comparison — also vulnerable.
    pub fn vulnerable_greater_equal_check(env: Env, nonce: u32) {
        if env.ledger().sequence() >= nonce {
            // Handle without storing
        }
    }
}