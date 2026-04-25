#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct StorageTypeConfusionSafe;

/// Safe: each key is always written with the same type.
#[contractimpl]
impl StorageTypeConfusionSafe {
    pub fn set_owner(env: Env, owner: Address) {
        env.storage().instance().set(&"owner", &owner);
    }

    pub fn update_owner(env: Env, new_owner: Address) {
        // Safe: same key "owner" always stores an Address.
        env.storage().instance().set(&"owner", &new_owner);
    }
}
