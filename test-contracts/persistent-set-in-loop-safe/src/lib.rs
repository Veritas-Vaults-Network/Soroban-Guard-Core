#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct PersistentSetInLoopSafe;

const KEY: Symbol = symbol_short!("key");

#[contractimpl]
impl PersistentSetInLoopSafe {
    /// Writes to persistent storage inside a loop with extend_ttl — should pass.
    pub fn batch_set(env: Env, count: u32) {
        for i in 0..count {
            let key = Symbol::new(&env, &format!("key_{}", i));
            env.storage().persistent().set(&key, &i);
            env.storage().persistent().extend_ttl(&key, 100, 200);
        }
    }
}
