#![no_std]
use soroban_sdk::{contract, contractimpl, token, Address, Env};

#[contract]
pub struct TokenTransferUncheckedSafe;

#[contractimpl]
impl TokenTransferUncheckedSafe {
    /// Binds the transfer result to a variable -- should pass the check.
    pub fn pay(env: Env, token_addr: Address, from: Address, to: Address, amount: i128) {
        let client = token::Client::new(&env, &token_addr);
        let _result = client.transfer(&from, &to, &amount);
    }
}
