#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol};

#[contract]
pub struct AdminInTempVulnerable;

const ADMIN: Symbol = symbol_short!("admin");

#[contractimpl]
impl AdminInTempVulnerable {
    /// ❌ Stores admin in temporary storage — expires with TTL, leaving contract without admin.
    pub fn set_admin(env: Env, new_admin: Address) {
        env.storage().temporary().set(&ADMIN, &new_admin);
    }
}
