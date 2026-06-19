#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct VulnerableContract;

const KEY: u32 = 0;

#[contractimpl]
impl VulnerableContract {
    pub fn store(env: Env, val: i128) {
        // ❌ Writes to persistent storage but never calls extend_ttl — entry may expire
        env.storage().persistent().set(&KEY, &val);
    }
}
