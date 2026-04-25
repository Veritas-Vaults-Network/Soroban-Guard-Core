#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

#[contract]
pub struct OwnershipTransferSafe;

/// Safe: reads and verifies the pending_owner sentinel before writing the new admin.
#[contractimpl]
impl OwnershipTransferSafe {
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

    pub fn accept_ownership(env: Env) {
        // Safe: read the pending owner first and verify the caller.
        let pending: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("pending"))
            .unwrap();
        pending.require_auth();
        env.storage()
            .instance()
            .set(&symbol_short!("admin"), &pending);
        env.storage()
            .instance()
            .remove(&symbol_short!("pending"));
    }
}
