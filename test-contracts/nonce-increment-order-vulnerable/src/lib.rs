#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, token, Address, Env};

#[contract]
pub struct NonceContract;

#[contractimpl]
impl NonceContract {
    /// ❌ Token transfer happens BEFORE nonce is written — replay risk.
    pub fn execute(env: Env, token_id: Address, to: Address, amount: i128) {
        let nonce: u64 = env
            .storage()
            .persistent()
            .get(&symbol_short!("nonce"))
            .unwrap_or(0);
        // external call before nonce update
        let client = token::Client::new(&env, &token_id);
        client.transfer(&env.current_contract_address(), &to, &amount);
        env.storage()
            .persistent()
            .set(&symbol_short!("nonce"), &(nonce + 1));
    }
}
