#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct VestingCliffVulnerable;

#[contractimpl]
impl VestingCliffVulnerable {
    pub fn initialize(env: Env, start_time: u64, cliff: u64) {
        env.storage().instance().set(&symbol_short!("start"), &start_time);
        env.storage().instance().set(&symbol_short!("cliff"), &cliff);
    }

    // ❌ cliff is a duration but compared directly against absolute timestamp
    // Should be: start_time + cliff <= now
    pub fn claim(env: Env) {
        let start_time: u64 = env.storage().instance().get(&symbol_short!("start")).unwrap();
        let cliff: u64 = env.storage().instance().get(&symbol_short!("cliff")).unwrap();
        let now = env.ledger().timestamp();
        // BUG: cliff is a duration (e.g. 90 days in seconds), not an absolute time
        assert!(cliff <= now, "cliff not reached");
        let _ = start_time;
        // ... distribute tokens
    }
}
