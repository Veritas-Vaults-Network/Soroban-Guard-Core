#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct MixedStorageTiersSafe;

const PERSISTENT_KEY: Symbol = symbol_short!("persist");
const INSTANCE_KEY: Symbol = symbol_short!("instance");

#[contractimpl]
impl MixedStorageTiersSafe {
    pub fn set_persistent(env: Env, amount: i128) {
        // ✅ Consistent use of persistent storage
        env.storage().persistent().set(&PERSISTENT_KEY, &amount);
    }

    pub fn set_instance(env: Env, amount: i128) {
        // ✅ Consistent use of instance storage with different key
        env.storage().instance().set(&INSTANCE_KEY, &amount);
    }

    pub fn get_persistent(env: Env) -> i128 {
        env.storage().persistent().get(&PERSISTENT_KEY).unwrap_or(0)
    }

    pub fn get_instance(env: Env) -> i128 {
        env.storage().instance().get(&INSTANCE_KEY).unwrap_or(0)
    }
}
