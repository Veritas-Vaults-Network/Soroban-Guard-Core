#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol};

#[contract]
pub struct AdminInTempSafe;

const ADMIN: Symbol = symbol_short!("admin");

#[contractimpl]
impl AdminInTempSafe {
    /// ✅ Admin stored in persistent storage — survives TTL expiry.
    pub fn set_admin(env: Env, new_admin: Address) {
        env.require_auth();
        env.storage().persistent().set(&ADMIN, &new_admin);
    }
}
