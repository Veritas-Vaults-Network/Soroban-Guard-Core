#![no_std]
use soroban_sdk::{contract, contractimpl, token, Address, Env};

#[contract]
pub struct TokenTransferUncheckedVulnerable;

#[contractimpl]
impl TokenTransferUncheckedVulnerable {
    /// Calls token::Client::transfer but ignores the return value.
    /// If the sender has insufficient balance the contract silently proceeds,
    /// causing accounting errors -- should trigger the check.
    pub fn pay(env: Env, token_addr: Address, from: Address, to: Address, amount: i128) {
        let client = token::Client::new(&env, &token_addr);
        client.transfer(&from, &to, &amount);
    }
}
