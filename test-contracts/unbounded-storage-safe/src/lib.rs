#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Vec};

#[contract]
pub struct UnboundedStorageSafe;

const MAX_ITEMS: u32 = 1000;

#[contractimpl]
impl UnboundedStorageSafe {
    /// Push with size check — safe.
    pub fn add_item(env: Env, item: i32) -> Result<(), String> {
        let storage_key = soroban_sdk::Symbol::new(&env, "items");
        let mut items: Vec<i32> = env
            .storage()
            .instance()
            .get(&storage_key)
            .unwrap_or_else(|| Vec::new(&env));
        
        if items.len() >= MAX_ITEMS {
            return Err("Max items reached".to_string());
        }
        
        items.push_back(item);
        env.storage().instance().set(&storage_key, &items);
        Ok(())
    }

    /// Insert with size check — safe.
    pub fn add_to_map(env: Env, key: u32, value: i32) -> Result<(), String> {
        let storage_key = soroban_sdk::Symbol::new(&env, "data");
        let mut data: soroban_sdk::Map<u32, i32> = env
            .storage()
            .instance()
            .get(&storage_key)
            .unwrap_or_else(|| soroban_sdk::Map::new(&env));
        
        if data.len() >= MAX_ITEMS {
            return Err("Map is full".to_string());
        }
        
        data.insert(key, value);
        env.storage().instance().set(&storage_key, &data);
        Ok(())
    }

    /// Append with capacity check — safe.
    pub fn append_items(env: Env, more_items: Vec<i32>) -> Result<(), String> {
        let storage_key = soroban_sdk::Symbol::new(&env, "all_items");
        let mut items: Vec<i32> = env
            .storage()
            .instance()
            .get(&storage_key)
            .unwrap_or_else(|| Vec::new(&env));
        
        if items.len() + more_items.len() > MAX_ITEMS as usize {
            return Err("Would exceed max items".to_string());
        }
        
        let mut i = 0;
        while i < more_items.len() {
            items.push_back(more_items.get(i).unwrap());
            i += 1;
        }
        
        env.storage().instance().set(&storage_key, &items);
        Ok(())
    }
}
