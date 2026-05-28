#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct VulnerableContract;

#[contractimpl]
impl VulnerableContract {
    pub fn store(env: Env, val: u32) {
        let key = symbol_short!("val");
        env.storage().persistent().set(&key, &val);
        // ❌ max_ttl (100) is far below the 10_000 ledger threshold
        env.storage().persistent().extend_ttl(key, 50, 100);
    }

    pub fn refresh(env: Env) {
        // ❌ instance TTL hardcoded to a low value
        env.storage().instance().extend_ttl(50, 100);
    }
}
