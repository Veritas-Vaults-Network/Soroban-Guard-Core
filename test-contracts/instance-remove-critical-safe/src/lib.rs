#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Symbol};

#[contract]
pub struct InstanceRemoveCriticalSafe;

#[contractimpl]
impl InstanceRemoveCriticalSafe {
    /// Removes admin with auth — should pass `instance-remove-critical`.
    pub fn remove_admin(env: Env) {
        env.require_auth();
        env.storage().instance().remove(&Symbol::new(&env, "admin"));
    }
}
