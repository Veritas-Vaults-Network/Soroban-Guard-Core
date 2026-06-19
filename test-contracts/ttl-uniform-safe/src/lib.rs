#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct SafeContract;

const CONFIG_KEY: u32 = 1;
const SESSION_KEY: u32 = 2;

#[contractimpl]
impl SafeContract {
    // ✅ Config data gets a long TTL; session data gets a short TTL
    pub fn store_config(env: Env, val: u32) {
        env.storage().persistent().set(&CONFIG_KEY, &val);
        env.storage().persistent().extend_ttl(&CONFIG_KEY, 10000, 20000);
    }

    pub fn store_session(env: Env, val: u32) {
        env.storage().persistent().set(&SESSION_KEY, &val);
        env.storage().persistent().extend_ttl(&SESSION_KEY, 100, 200);
    }
}
