#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct VulnerableContract;

const DATA_KEY: soroban_sdk::Symbol = symbol_short!("data");
const GHOST_KEY: soroban_sdk::Symbol = symbol_short!("ghost");

#[contractimpl]
impl VulnerableContract {
    pub fn store(env: Env, val: u32) {
        env.storage().persistent().set(&DATA_KEY, &val);
        env.storage().persistent().extend_ttl(DATA_KEY, 10000, 17280);
    }

    pub fn refresh(env: Env) {
        // ❌ GHOST_KEY is never written via set — phantom extension
        env.storage().persistent().extend_ttl(GHOST_KEY, 10000, 17280);
    }
}
