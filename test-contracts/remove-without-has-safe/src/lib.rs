#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct SafeContract;

#[contractimpl]
impl SafeContract {
    pub fn remove_key(env: Env) {
        // ✅ Remove guarded by has() check
        if env.storage().instance().has(&symbol_short!("key")) {
            env.storage().instance().remove(&symbol_short!("key"));
        }
    }

    pub fn remove_persistent(env: Env) {
        // ✅ Remove from persistent storage with has() guard
        if env.storage().persistent().has(&Symbol::new(&env, "data")) {
            env.storage().persistent().remove(&Symbol::new(&env, "data"));
        }
    }
}
