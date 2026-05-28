#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct AdminNoEventVulnerable;

#[contractimpl]
impl AdminNoEventVulnerable {
    /// ❌ Changes admin without emitting an event — invisible to indexers.
    pub fn set_admin(env: Env, new_admin: Address) {
        new_admin.require_auth();
        env.storage().instance().set(&"admin", &new_admin);
    }

    /// ❌ Transfers ownership without emitting an event.
    pub fn transfer_ownership(env: Env, new_owner: Address) {
        new_owner.require_auth();
        env.storage().instance().set(&"owner", &new_owner);
    }
}
