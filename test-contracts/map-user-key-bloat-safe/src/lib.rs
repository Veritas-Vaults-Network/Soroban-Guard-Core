#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Map, Symbol};

#[contract]
pub struct MapUserKeyBloatSafe;

#[contractimpl]
impl MapUserKeyBloatSafe {
    /// Map::set with user key but has len guard — should pass `map-user-key-bloat`.
    pub fn add_item(env: Env, user_key: Symbol, value: u32) {
        let mut map: Map<Symbol, u32> = env.storage().instance().get(&Symbol::new(&env, "map")).unwrap_or(Map::new(&env));
        if map.len() >= 100 {
            panic!("Map is full");
        }
        map.set(user_key, value);
        env.storage().instance().set(&Symbol::new(&env, "map"), &map);
    }
}
