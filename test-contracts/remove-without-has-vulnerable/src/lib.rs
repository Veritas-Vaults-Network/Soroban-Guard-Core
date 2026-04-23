#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct VulnerableContract;

#[contractimpl]
impl VulnerableContract {
    pub fn remove_key(env: Env) {
        // ❌ Unconditional remove without has() guard
        env.storage().instance().remove(&symbol_short!("key"));
    }

    pub fn remove_persistent(env: Env) {
        // ❌ Remove from persistent storage without checking if key exists
        env.storage().persistent().remove(&Symbol::new(&env, "data"));
    }
}
