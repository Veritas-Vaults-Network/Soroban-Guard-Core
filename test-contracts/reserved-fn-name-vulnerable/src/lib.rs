#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct VulnerableContract;

#[contractimpl]
impl VulnerableContract {
    // ❌ __constructor is a Soroban SDK reserved name — unexpected dispatch behaviour
    pub fn __constructor(env: Env) {
        let _ = env;
    }
}
