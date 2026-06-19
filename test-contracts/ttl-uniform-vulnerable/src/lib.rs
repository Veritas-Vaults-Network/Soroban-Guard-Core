#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct VulnerableContract;

const CONFIG_KEY: u32 = 1;
const SESSION_KEY: u32 = 2;

#[contractimpl]
impl VulnerableContract {
    // ❌ Both keys use the same TTL (1000, 2000) regardless of criticality
    pub fn store_config(env: Env, val: u32) {
        env.storage().persistent().set(&CONFIG_KEY, &val);
        env.storage().persistent().extend_ttl(&CONFIG_KEY, 1000, 2000);
    }

    pub fn store_session(env: Env, val: u32) {
        env.storage().persistent().set(&SESSION_KEY, &val);
        env.storage().persistent().extend_ttl(&SESSION_KEY, 1000, 2000);
    }
}
