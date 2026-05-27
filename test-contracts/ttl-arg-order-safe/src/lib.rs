#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct SafeContract;

#[contractimpl]
impl SafeContract {
    pub fn extend_ttl_good(env: Env) {
        // ✅ Arguments in correct order: min_ttl (100) <= max_ttl (1000)
        env.storage().instance().extend_ttl(100, 1000);
    }

    pub fn extend_ttl_good_persistent(env: Env) {
        // ✅ Arguments in correct order for persistent storage
        env.storage().persistent().extend_ttl(50, 500);
    }
}
