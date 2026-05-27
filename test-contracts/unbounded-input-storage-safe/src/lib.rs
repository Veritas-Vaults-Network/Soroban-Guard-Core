#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Vec, Map, Symbol};

#[contract]
pub struct UnboundedInputStorageSafe;

const MAX_ITEMS: u32 = 1000;

#[contractimpl]
impl UnboundedInputStorageSafe {
    /// Vec parameter with size guard before storage set – should be safe.
    pub fn store_vec(env: Env, items: Vec<u32>) -> Result<(), &'static str> {
        if items.len() > MAX_ITEMS {
            return Err("Too many items");
        }
        let key = Symbol::new(&env, "items");
        env.storage().persistent().set(&key, &items);
        Ok(())
    }

    /// Map parameter with size guard before storage set – should be safe.
    pub fn store_map(env: Env, data: Map<u32, u32>) -> Result<(), &'static str> {
        if data.len() > MAX_ITEMS {
            return Err("Map too large");
        }
        let key = Symbol::new(&env, "data");
        env.storage().persistent().set(&key, &data);
        Ok(())
    }

    /// Vec parameter with guard on clone – still safe.
    pub fn store_vec_clone(env: Env, items: Vec<u32>) -> Result<(), &'static str> {
        if items.len() > MAX_ITEMS {
            return Err("Too many items");
        }
        let key = Symbol::new(&env, "items");
        let cloned = items.clone();
        env.storage().persistent().set(&key, &cloned);
        Ok(())
    }
}
