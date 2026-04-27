#![no_std]
use soroban_sdk::{contract, contractimpl, token, Address, Env};

#[contract]
pub struct TransferToSelfSafe;

#[contractimpl]
impl TransferToSelfSafe {
    // ✅ Recipient verified not to be the contract itself
    pub fn send(env: Env, token: Address, to: Address, amount: i128) {
        assert!(to != env.current_contract_address(), "cannot transfer to self");
        let client = token::Client::new(&env, &token);
        let contract_addr = env.current_contract_address();
        client.transfer(&contract_addr, &to, &amount);
    }
}
