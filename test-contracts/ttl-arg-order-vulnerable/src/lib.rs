#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct VulnerableContract;

#[contractimpl]
impl VulnerableContract {
    pub fn extend_ttl_bad(env: Env) {
        // ❌ Arguments are swapped: min_ttl (1000) > max_ttl (100)
        env.storage().instance().extend_ttl(1000, 100);
    }

    pub fn extend_ttl_bad_persistent(env: Env) {
        // ❌ Arguments are swapped in persistent storage
        env.storage().persistent().extend_ttl(500, 50);
    }
}
