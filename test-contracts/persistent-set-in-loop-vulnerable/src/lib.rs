#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct PersistentSetInLoopVulnerable;

const KEY: Symbol = symbol_short!("key");

#[contractimpl]
impl PersistentSetInLoopVulnerable {
    /// Writes to persistent storage inside a loop without extend_ttl — should trigger check.
    pub fn batch_set(env: Env, count: u32) {
        for i in 0..count {
            let key = Symbol::new(&env, &format!("key_{}", i));
            env.storage().persistent().set(&key, &i);
        }
    }
}
