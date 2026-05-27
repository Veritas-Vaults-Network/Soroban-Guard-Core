#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol};

#[contract]
pub struct AuthAfterWriteVulnerable;

const KEY: Symbol = symbol_short!("owner");

#[contractimpl]
impl AuthAfterWriteVulnerable {
    /// Calls require_auth after storage write — should trigger check.
    pub fn set_owner(env: Env, new_owner: Address) {
        env.storage().instance().set(&KEY, &new_owner);
        env.require_auth();
    }
}
