#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct VulnerableContract;

#[contractimpl]
impl VulnerableContract {
    pub fn bad_method(env: Env) {
        // ❌ Incorrect: env.current_contract_address() returns Address directly, not Option
        let addr = env.current_contract_address().unwrap();
    }
}