#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct MixedStorageTiersVulnerable;

const KEY: Symbol = symbol_short!("balance");

#[contractimpl]
impl MixedStorageTiersVulnerable {
    pub fn set_balance(env: Env, amount: i128) {
        // ❌ Same key written to multiple storage tiers
        env.storage().persistent().set(&KEY, &amount);
        env.storage().instance().set(&KEY, &amount);
    }

    pub fn get_balance(env: Env) -> i128 {
        // ❌ Reading from different tier than where it was written
        env.storage().persistent().get(&KEY).unwrap_or(0)
    }
}
