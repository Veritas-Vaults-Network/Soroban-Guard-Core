#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Map};

#[contract]
pub struct MapKeyExplosionVulnerable;

#[contractimpl]
impl MapKeyExplosionVulnerable {
    /// Inserts more than 8 distinct string literal keys into a Map.
    pub fn vulnerable_key_explosion(env: Env) {
        let mut map = Map::new(&env);
        map.set("key1", 1);
        map.set("key2", 2);
        map.set("key3", 3);
        map.set("key4", 4);
        map.set("key5", 5);
        map.set("key6", 6);
        map.set("key7", 7);
        map.set("key8", 8);
        map.set("key9", 9);
    }
}