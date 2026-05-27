#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct StorageTypeConfusionVulnerable;

/// Vulnerable: the same key "owner" is written with an Address in one function
/// and with an i128 in another. Readers will deserialize garbage.
#[contractimpl]
impl StorageTypeConfusionVulnerable {
    pub fn set_owner(env: Env, owner: Address) {
        // Writes Address under "owner".
        env.storage().instance().set(&"owner", &owner);
    }

    pub fn reset_owner_id(env: Env) {
        // BUG: writes i128 under the same "owner" key.
        env.storage().instance().set(&"owner", &0i128);
    }
}
