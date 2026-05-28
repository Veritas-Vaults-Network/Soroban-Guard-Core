#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol};

#[contract]
pub struct TempForPersistentDataVulnerable;

const ADMIN: Symbol = symbol_short!("admin");
const TOTAL_SUPPLY: Symbol = symbol_short!("total_supply");

#[contractimpl]
impl TempForPersistentDataVulnerable {
    /// ❌ Stores admin in temporary storage — expires with TTL, causing permanent data loss.
    pub fn set_admin(env: Env, new_admin: Address) {
        env.storage().temporary().set(&ADMIN, &new_admin);
    }

    /// ❌ Stores total_supply in temporary storage — expires with TTL.
    pub fn init_supply(env: Env, supply: i128) {
        env.storage().temporary().set(&TOTAL_SUPPLY, &supply);
    }
}
