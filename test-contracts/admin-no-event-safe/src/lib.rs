#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

#[contract]
pub struct AdminNoEventSafe;

#[contractimpl]
impl AdminNoEventSafe {
    /// ✅ Emits an event after changing admin so indexers can track the change.
    pub fn set_admin(env: Env, new_admin: Address) {
        new_admin.require_auth();
        env.storage().instance().set(&"admin", &new_admin);
        env.events().publish((symbol_short!("set_admin"),), new_admin);
    }

    /// ✅ Emits an event after transferring ownership.
    pub fn transfer_ownership(env: Env, new_owner: Address) {
        new_owner.require_auth();
        env.storage().instance().set(&"owner", &new_owner);
        env.events().publish((symbol_short!("new_owner"),), new_owner);
    }
}
