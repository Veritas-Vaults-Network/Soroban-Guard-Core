#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct TokenContract;

#[contractimpl]
impl TokenContract {
    // ✅ Asserts from != to before proceeding.
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        assert!(from != to, "self-transfer not allowed");
        from.require_auth();
        let _ = (env, amount);
    }
}
