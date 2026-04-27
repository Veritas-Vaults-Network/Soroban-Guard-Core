#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct UnauthStorageRemoveVulnerable;

#[contractimpl]
impl UnauthStorageRemoveVulnerable {
    /// ❌ Any caller can pass any `user` address and delete their balance entry.
    /// No `user.require_auth()` or `env.require_auth()` is called first.
    pub fn clear_balance(env: Env, user: Address) {
        env.storage().persistent().remove(&user);
    }
}
