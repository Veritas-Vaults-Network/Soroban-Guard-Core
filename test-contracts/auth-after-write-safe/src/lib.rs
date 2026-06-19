#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol};

#[contract]
pub struct AuthAfterWriteSafe;

const KEY: Symbol = symbol_short!("owner");

#[contractimpl]
impl AuthAfterWriteSafe {
    /// Calls require_auth before storage write — should pass.
    pub fn set_owner(env: Env, new_owner: Address) {
        env.require_auth();
        env.storage().instance().set(&KEY, &new_owner);
    }
}
