#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct AdminKeyRemovalVulnerable;

const ADMIN_KEY: soroban_sdk::Symbol = symbol_short!("admin");

#[contractimpl]
impl AdminKeyRemovalVulnerable {
    /// ❌ Removes the admin key without replacing it — contract is permanently
    /// left without an admin after this call.
    pub fn remove_admin(env: Env) {
        env.storage().persistent().remove(&ADMIN_KEY);
    }
}
