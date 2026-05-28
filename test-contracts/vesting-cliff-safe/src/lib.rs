#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct VestingCliffSafe;

#[contractimpl]
impl VestingCliffSafe {
    pub fn initialize(env: Env, start_time: u64, cliff: u64) {
        env.storage().instance().set(&symbol_short!("start"), &start_time);
        env.storage().instance().set(&symbol_short!("cliff"), &cliff);
    }

    // ✅ cliff duration added to start_time before comparing against timestamp
    pub fn claim(env: Env) {
        let start_time: u64 = env.storage().instance().get(&symbol_short!("start")).unwrap();
        let cliff: u64 = env.storage().instance().get(&symbol_short!("cliff")).unwrap();
        let now = env.ledger().timestamp();
        assert!(start_time + cliff <= now, "cliff not reached");
        // ... distribute tokens
    }
}
