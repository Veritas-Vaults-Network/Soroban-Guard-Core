#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    // ✅ Panics with a clear message if admin is not set — no silent zero-address.
    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get::<_, Address>(&symbol_short!("admin"))
            .expect("admin not initialized")
    }

    // ✅ Returns Option so the caller decides how to handle absence.
    pub fn try_get_recipient(env: Env) -> Option<Address> {
        env.storage()
            .persistent()
            .get::<_, Address>(&symbol_short!("recip"))
    }

    // ✅ unwrap_or_default on a non-Address type is fine.
    pub fn get_count(env: Env) -> u32 {
        env.storage()
            .instance()
            .get::<_, u32>(&symbol_short!("count"))
            .unwrap_or_default()
    }
}
