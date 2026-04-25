#![no_std]

use soroban_sdk::{contract, contractimpl, Env};

pub struct StoredEnv {
    env: Env,
}

#[contract]
pub struct EnvInStructVulnerable;

#[contractimpl]
impl EnvInStructVulnerable {
    pub fn test(_env: Env) {
        let _ = 1u32;
    }
}
