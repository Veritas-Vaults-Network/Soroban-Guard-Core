#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol};

#[contract]
pub struct RedundantAuthArgsSafe;

const ADMIN: Symbol = symbol_short!("admin");

#[contractimpl]
impl RedundantAuthArgsSafe {
    /// ✅ Args include call-specific parameters.
    pub fn good_auth(env: Env, user: Address, amount: i128) {
        env.require_auth_for_args((user, amount));
    }

    /// ✅ Args include function parameters that are call-specific.
    pub fn transfer(env: Env, recipient: Address, amount: i128) {
        env.require_auth_for_args((recipient, amount));
    }
}
