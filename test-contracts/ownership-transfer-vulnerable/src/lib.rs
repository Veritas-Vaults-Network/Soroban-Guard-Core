#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

#[contract]
pub struct OwnershipTransferVulnerable;

/// Vulnerable: writes the new admin without reading the pending_owner sentinel.
/// Any address can call accept_ownership and become the admin.
#[contractimpl]
impl OwnershipTransferVulnerable {
    pub fn propose_owner(env: Env, new_owner: Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .unwrap();
        admin.require_auth();
        env.storage()
            .instance()
            .set(&symbol_short!("pending"), &new_owner);
    }

    pub fn accept_ownership(env: Env, caller: Address) {
        // BUG: does not read pending_owner before writing admin.
        env.storage()
            .instance()
            .set(&symbol_short!("admin"), &caller);
    }
}
