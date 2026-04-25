#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct AssertForAuthSafe;

#[contractimpl]
impl AssertForAuthSafe {
    /// Uses require_auth for proper access control — safe.
    pub fn safe_auth(env: Env, caller: Address) {
        env.require_auth(&caller);
        // Do privileged operation
    }

    /// Uses require_auth_for_args for proper authorization — safe.
    pub fn safe_auth_for_args(env: Env, caller: Address, amount: i128) {
        env.require_auth_for_args((caller, amount));
        // Do operation
    }

    /// Uses assert! for non-auth purposes — safe.
    pub fn safe_assert(env: Env) {
        assert!(1 + 1 == 2);
        // Non-auth assertion
    }
}