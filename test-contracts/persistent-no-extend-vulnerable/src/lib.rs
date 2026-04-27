#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

#[contract]
pub struct VulnerableContract;

#[contractimpl]
impl VulnerableContract {
    // ❌ Persistent entry written but extend_ttl never called — will expire
    pub fn initialize(env: Env, admin: Address) {
        env.storage()
            .persistent()
            .set(&symbol_short!("admin"), &admin);
    }

    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .persistent()
            .get(&symbol_short!("admin"))
            .unwrap()
    }
}
