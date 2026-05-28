#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

#[contract]
pub struct AdminKeyRemovalSafe;

const ADMIN_KEY: soroban_sdk::Symbol = symbol_short!("admin");

#[contractimpl]
impl AdminKeyRemovalSafe {
    /// ✅ Atomically replaces the admin key — remove is immediately followed by
    /// set in the same transaction, so the contract always has an admin.
    pub fn rotate_admin(env: Env, new_admin: Address) {
        env.storage().persistent().remove(&ADMIN_KEY);
        env.storage().persistent().set(&ADMIN_KEY, &new_admin);
    }
}
