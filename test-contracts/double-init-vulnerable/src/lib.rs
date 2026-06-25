#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, Symbol};

#[contract]
pub struct DoubleInitVulnerable;

#[contractimpl]
impl DoubleInitVulnerable {
    /// Re-initializes owner storage every time it is called.
    pub fn initialize(env: Env, owner: Address) {
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "owner"), &owner);
    }
}
