#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct U32TimestampVulnerable;

#[contractimpl]
impl U32TimestampVulnerable {
    // ❌ u32 truncates Soroban ledger timestamps after 2038-01-19.
    pub fn set_deadline(env: Env, deadline: u32) {
        let _ = (env, deadline);
    }

    pub fn create_offer(env: Env, timestamp: u32, expiry: u32) {
        let _ = (env, timestamp, expiry);
    }

    pub fn schedule(env: Env, time: u32, expiration: u32) {
        let _ = (env, time, expiration);
    }
}
