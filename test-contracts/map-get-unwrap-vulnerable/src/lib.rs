#![no_std]

use soroban_sdk::{contract, contractimpl, Env, Map};

#[contract]
pub struct MapGetUnwrapVulnerable;

#[contractimpl]
impl MapGetUnwrapVulnerable {
    // ❌ No map.has(&key) guard — panics if key is absent
    pub fn get_value(env: Env, map: Map<u32, u32>, key: u32) -> u32 {
        map.get(&key).unwrap()
    }
}
