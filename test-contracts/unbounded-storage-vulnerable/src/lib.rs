#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Vec};

#[contract]
pub struct UnboundedStorageVulnerable;

#[contractimpl]
impl UnboundedStorageVulnerable {
    /// Unbounded push_back without size check — should trigger `unbounded-storage` (Medium).
    pub fn add_item(env: Env, item: i32) {
        let storage_key = soroban_sdk::Symbol::new(&env, "items");
        let mut items: Vec<i32> = env
            .storage()
            .instance()
            .get(&storage_key)
            .unwrap_or_else(|| Vec::new(&env));
        items.push_back(item);
        env.storage().instance().set(&storage_key, &items);
    }

    /// Unbounded insert without size check — should trigger `unbounded-storage` (Medium).
    pub fn add_to_map(env: Env, key: u32, value: i32) {
        let storage_key = soroban_sdk::Symbol::new(&env, "data");
        let mut data: soroban_sdk::Map<u32, i32> = env
            .storage()
            .instance()
            .get(&storage_key)
            .unwrap_or_else(|| soroban_sdk::Map::new(&env));
        data.insert(key, value);
        env.storage().instance().set(&storage_key, &data);
    }

    /// Another unbounded operation.
    pub fn append_items(env: Env, more_items: Vec<i32>) {
        let storage_key = soroban_sdk::Symbol::new(&env, "all_items");
        let mut items: Vec<i32> = env
            .storage()
            .instance()
            .get(&storage_key)
            .unwrap_or_else(|| Vec::new(&env));
        
        let mut i = 0;
        while i < more_items.len() {
            items.push_back(more_items.get(i).unwrap());
            i += 1;
        }
        
        env.storage().instance().set(&storage_key, &items);
    }
}
