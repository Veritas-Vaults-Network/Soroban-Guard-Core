#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

#[contract]
pub struct OwnershipImmediateSafe;

#[contractimpl]
impl OwnershipImmediateSafe {
    /// ✅ Step 1: propose a new admin (stores in pending slot, does not transfer yet).
    pub fn set_admin(env: Env, new_admin: Address) {
        let current: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .unwrap();
        current.require_auth();
        env.storage()
            .instance()
            .set(&symbol_short!("pending"), &new_admin);
    }

    /// ✅ Step 2: the pending admin must accept — two-step handoff prevents
    /// accidental permanent loss of ownership.
    pub fn accept_admin(env: Env) {
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
