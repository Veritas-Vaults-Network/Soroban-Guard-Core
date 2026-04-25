#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Vec, Map, Symbol};

#[contract]
pub struct UnboundedInputStorageVulnerable;

#[contractimpl]
impl UnboundedInputStorageVulnerable {
    /// Unbounded Vec parameter written directly to storage – should trigger
    /// `unbounded-input-storage` (High).
    pub fn store_vec(env: Env, items: Vec<u32>) {
        let key = Symbol::new(&env, "items");
        env.storage().persistent().set(&key, &items);
    }

    /// Unbounded Map parameter written directly to storage – should trigger
    /// `unbounded-input-storage` (High).
    pub fn store_map(env: Env, data: Map<u32, u32>) {
        let key = Symbol::new(&env, "data");
        env.storage().persistent().set(&key, &data);
    }

    /// Vec parameter used after cloning (still unbounded) – should trigger.
    pub fn store_vec_clone(env: Env, items: Vec<u32>) {
        let key = Symbol::new(&env, "items");
        let cloned = items.clone();
        env.storage().persistent().set(&key, &cloned);
    }
}
