#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct SafeContract;

#[contractimpl]
impl SafeContract {
    /// ✅ require_auth() is called on the caller parameter — real access control.
    pub fn admin_action(env: Env, caller: Address) {
        caller.require_auth();
        // ... privileged logic
    }
}
