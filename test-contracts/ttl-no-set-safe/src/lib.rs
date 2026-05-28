#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct SafeContract;

const DATA_KEY: soroban_sdk::Symbol = symbol_short!("data");

#[contractimpl]
impl SafeContract {
    pub fn store(env: Env, val: u32) {
        // ✅ DATA_KEY is written via set before extend_ttl
        env.storage().persistent().set(&DATA_KEY, &val);
        env.storage().persistent().extend_ttl(DATA_KEY, 10000, 17280);
    }

    pub fn refresh(env: Env) {
        // ✅ DATA_KEY is set elsewhere in the contract
        env.storage().persistent().extend_ttl(DATA_KEY, 10000, 17280);
    }
}
