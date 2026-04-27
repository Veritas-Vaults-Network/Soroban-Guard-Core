#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Map, Symbol};

#[contract]
pub struct MapUserKeyBloatVulnerable;

#[contractimpl]
impl MapUserKeyBloatVulnerable {
    /// Map::set with user key, no len guard — should trigger `map-user-key-bloat` (Medium).
    pub fn add_item(env: Env, user_key: Symbol, value: u32) {
        let mut map: Map<Symbol, u32> = env.storage().instance().get(&Symbol::new(&env, "map")).unwrap_or(Map::new(&env));
        map.set(user_key, value);
        env.storage().instance().set(&Symbol::new(&env, "map"), &map);
    }
}
