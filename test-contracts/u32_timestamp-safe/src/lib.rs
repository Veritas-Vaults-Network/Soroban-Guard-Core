#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct U32TimestampSafe;

#[contractimpl]
impl U32TimestampSafe {
    // ✅ u64 matches the Soroban ledger timestamp type — no Year 2038 truncation.
    pub fn set_deadline(env: Env, deadline: u64) {
        let _ = (env, deadline);
    }

    pub fn create_offer(env: Env, timestamp: u64, expiry: u64) {
        let _ = (env, timestamp, expiry);
    }

    pub fn schedule(env: Env, time: u64, expiration: u64) {
        let _ = (env, time, expiration);
    }

    // ✅ u32 is fine for non-timestamp values.
    pub fn set_count(env: Env, count: u32) {
        let _ = (env, count);
    }
}
