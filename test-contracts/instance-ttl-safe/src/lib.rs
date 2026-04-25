#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct SafeContract;

const KEY: u32 = 0;

#[contractimpl]
impl SafeContract {
    pub fn store(env: Env, val: i128) {
        // ✅ Uses instance storage AND extends TTL to prevent expiration
        env.storage().instance().set(&KEY, &val);
        env.storage().instance().extend_ttl(1000, 2000);
    }
}