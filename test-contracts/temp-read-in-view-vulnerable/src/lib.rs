#![no_std]

use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct TempReadInViewVulnerable;

#[contractimpl]
impl TempReadInViewVulnerable {
    pub fn get(env: Env, key: u32) -> Option<u32> {
        env.storage().temporary().get(&key)
    }
}
