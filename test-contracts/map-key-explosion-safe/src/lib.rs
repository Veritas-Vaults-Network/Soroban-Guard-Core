#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Map};

#[contract]
pub struct MapKeyExplosionSafe;

#[derive(Clone)]
#[contracttype]
pub enum Key {
    First,
    Second,
    Third,
}

#[contractimpl]
impl MapKeyExplosionSafe {
    /// Uses typed enum keys instead of excessive string literals — safe approach.
    /// This maintains type safety and makes storage layout auditable.
    pub fn safe_typed_keys(env: Env) {
        let mut map = Map::new(&env);
        map.set(Key::First, 1);
        map.set(Key::Second, 2);
        map.set(Key::Third, 3);
    }

    /// Uses exactly 8 distinct string literal keys — within safe threshold.
    /// While not ideal, this stays under the key explosion limit.
    pub fn safe_few_string_keys(env: Env) {
        let mut map = Map::new(&env);
        map.set("key1", 1);
        map.set("key2", 2);
        map.set("key3", 3);
        map.set("key4", 4);
        map.set("key5", 5);
        map.set("key6", 6);
        map.set("key7", 7);
        map.set("key8", 8);
    }
}