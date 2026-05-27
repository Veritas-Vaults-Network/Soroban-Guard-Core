#![no_std]

use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct TempGetNoHasVulnerable;

#[contractimpl]
impl TempGetNoHasVulnerable {
    pub fn get_without_check(env: Env, key: u32) -> Option<u32> {
        env.storage().temporary().get(&key)
    }
}
