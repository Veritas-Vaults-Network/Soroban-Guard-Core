#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct VulnerableContract;

const KEY: u32 = 0;

#[contractimpl]
impl VulnerableContract {
    pub fn store(env: Env, val: i128) {
        // ❌ Uses instance storage but never calls extend_ttl — contract may become inaccessible
        env.storage().instance().set(&KEY, &val);
    }
}