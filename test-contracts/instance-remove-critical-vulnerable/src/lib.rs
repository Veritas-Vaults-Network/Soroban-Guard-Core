#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Symbol};

#[contract]
pub struct InstanceRemoveCriticalVulnerable;

#[contractimpl]
impl InstanceRemoveCriticalVulnerable {
    /// Removes admin without auth — should trigger `instance-remove-critical` (High).
    pub fn remove_admin(env: Env) {
        env.storage().instance().remove(&Symbol::new(&env, "admin"));
    }
}
