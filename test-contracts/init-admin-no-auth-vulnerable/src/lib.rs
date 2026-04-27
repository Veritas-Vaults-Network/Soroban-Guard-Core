#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

#[contract]
pub struct VulnerableContract;

#[contractimpl]
impl VulnerableContract {
    // ❌ No require_auth — any caller can become admin on first call
    pub fn initialize(env: Env, admin: Address) {
        env.storage()
            .instance()
            .set(&symbol_short!("admin"), &admin);
    }
}
