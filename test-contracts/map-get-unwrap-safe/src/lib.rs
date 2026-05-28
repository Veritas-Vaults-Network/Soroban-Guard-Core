#![no_std]

use soroban_sdk::{contract, contractimpl, Env, Map};

#[contract]
pub struct MapGetUnwrapSafe;

#[contractimpl]
impl MapGetUnwrapSafe {
    // ✅ has() guard before get().unwrap()
    pub fn get_value(env: Env, map: Map<u32, u32>, key: u32) -> u32 {
        if map.has(&key) {
            map.get(&key).unwrap()
        } else {
            0
        }
    }
}
