#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct SafeContract;

#[contractimpl]
impl SafeContract {
    pub fn good_method(env: Env) {
        // ✅ Correct: env.current_contract_address() returns Address directly
        let addr = env.current_contract_address();
    }
}