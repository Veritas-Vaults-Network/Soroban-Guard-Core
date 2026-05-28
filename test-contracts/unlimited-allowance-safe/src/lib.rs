#![no_std]
use soroban_sdk::{contract, contractimpl, token, Address, Env};

#[contract]
pub struct UnlimitedAllowanceSafe;

#[contractimpl]
impl UnlimitedAllowanceSafe {
    /// Approves a caller-supplied bounded amount — safe.
    pub fn approve(env: Env, from: Address, spender: Address, token_id: Address, amount: i128) {
        let token = token::Client::new(&env, &token_id);
        token.approve(&from, &spender, &amount, &999999);
    }
}
