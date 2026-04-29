#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, token, Address, Env};

#[contract]
pub struct NonceContract;

#[contractimpl]
impl NonceContract {
    /// ✅ Nonce written BEFORE external call — no replay window.
    pub fn execute(env: Env, token_id: Address, to: Address, amount: i128) {
        let nonce: u64 = env
            .storage()
            .persistent()
            .get(&symbol_short!("nonce"))
            .unwrap_or(0);
        // increment nonce first
        env.storage()
            .persistent()
            .set(&symbol_short!("nonce"), &(nonce + 1));
        let client = token::Client::new(&env, &token_id);
        client.transfer(&env.current_contract_address(), &to, &amount);
    }
}
