#![no_std]
use soroban_sdk::{contract, contractimpl, token, Address, Env};

#[contract]
pub struct UnlimitedAllowanceVulnerable;

#[contractimpl]
impl UnlimitedAllowanceVulnerable {
    /// Approves i128::MAX — unlimited approval anti-pattern.
    /// Should trigger `unlimited-allowance` (Medium).
    pub fn approve_max(env: Env, from: Address, spender: Address, token_id: Address) {
        let token = token::Client::new(&env, &token_id);
        token.approve(&from, &spender, &i128::MAX, &999999);
    }

    /// Approves using the raw i128::MAX literal.
    /// Should trigger `unlimited-allowance` (Medium).
    pub fn approve_max_literal(env: Env, from: Address, spender: Address, token_id: Address) {
        let token = token::Client::new(&env, &token_id);
        token.approve(
            &from,
            &spender,
            &170141183460469231731687303715884105727_i128,
            &999999,
        );
    }
}
