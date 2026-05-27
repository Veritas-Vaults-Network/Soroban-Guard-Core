#![no_std]

use soroban_sdk::{contract, contractimpl, token, Address, Env};

#[contract]
pub struct AmountMulOverflowVulnerable;

#[contractimpl]
impl AmountMulOverflowVulnerable {
    pub fn pay(env: Env, token_id: Address, from: Address, to: Address, price: i128, quantity: i128) {
        let client = token::Client::new(&env, &token_id);
        client.transfer(&from, &to, &(price * quantity));
    }
}
