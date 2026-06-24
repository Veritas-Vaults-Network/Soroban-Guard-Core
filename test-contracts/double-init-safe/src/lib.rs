#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, Symbol};

#[contract]
pub struct DoubleInitSafe;

#[contractimpl]
impl DoubleInitSafe {
    /// Checks owner storage before writing one-time initialization state.
    pub fn initialize(env: Env, owner: Address) {
        let owner_key = Symbol::new(&env, "owner");
        if env.storage().instance().has(&owner_key) {
            panic!("already initialized");
        }

        env.storage().instance().set(&owner_key, &owner);
    }
}
