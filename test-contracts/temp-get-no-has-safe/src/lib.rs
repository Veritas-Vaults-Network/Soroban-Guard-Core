#![no_std]

use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct TempGetNoHasSafe;

#[contractimpl]
impl TempGetNoHasSafe {
    pub fn get_with_check(env: Env, key: u32) -> Option<u32> {
        if env.storage().temporary().has(&key) {
            Some(env.storage().temporary().get(&key))
        } else {
            None
        }
    }
}
