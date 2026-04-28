#![no_std]

use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct TempReadInViewSafe;

#[contractimpl]
impl TempReadInViewSafe {
    pub fn get(env: Env, key: u32) -> Option<u32> {
        if env.storage().temporary().has(&key) {
            Some(env.storage().temporary().get(&key))
        } else {
            None
        }
    }
}
