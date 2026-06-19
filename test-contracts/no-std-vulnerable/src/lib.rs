// ❌ Missing #![no_std] and uses std:: paths — will fail on WASM targets.
use std::collections::HashMap;
use soroban_sdk::{contract, contractimpl, Env, Symbol};

#[contract]
pub struct BadContract;

#[contractimpl]
impl BadContract {
    pub fn store(_env: Env) {
        let _map: HashMap<u32, u32> = HashMap::new();
    }
}
