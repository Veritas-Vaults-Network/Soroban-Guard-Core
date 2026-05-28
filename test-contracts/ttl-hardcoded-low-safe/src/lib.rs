#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct SafeContract;

#[contractimpl]
impl SafeContract {
    pub fn store(env: Env, val: u32) {
        let key = symbol_short!("val");
        env.storage().persistent().set(&key, &val);
        // ✅ max_ttl (17280) is above the 10_000 ledger threshold (~1 day)
        env.storage().persistent().extend_ttl(key, 10000, 17280);
    }

    pub fn refresh(env: Env) {
        // ✅ instance TTL set to a reasonable value
        env.storage().instance().extend_ttl(10000, 17280);
    }
}
