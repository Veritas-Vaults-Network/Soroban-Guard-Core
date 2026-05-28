#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol};

#[contract]
pub struct TempForPersistentDataSafe;

const ADMIN: Symbol = symbol_short!("admin");
const TOTAL_SUPPLY: Symbol = symbol_short!("total_supply");

#[contractimpl]
impl TempForPersistentDataSafe {
    /// ✅ Stores admin in persistent storage — survives TTL.
    pub fn set_admin(env: Env, new_admin: Address) {
        env.require_auth();
        env.storage().persistent().set(&ADMIN, &new_admin);
    }

    /// ✅ Stores total_supply in instance storage — survives TTL.
    pub fn init_supply(env: Env, supply: i128) {
        env.storage().instance().set(&TOTAL_SUPPLY, &supply);
    }
}
