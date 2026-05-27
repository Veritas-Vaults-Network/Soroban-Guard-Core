#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct UnauthStorageRemoveSafe;

#[contractimpl]
impl UnauthStorageRemoveSafe {
    /// ✅ `user.require_auth()` ensures only the owner of that address can
    /// delete their own balance entry.
    pub fn clear_balance(env: Env, user: Address) {
        user.require_auth();
        env.storage().persistent().remove(&user);
    }
}
