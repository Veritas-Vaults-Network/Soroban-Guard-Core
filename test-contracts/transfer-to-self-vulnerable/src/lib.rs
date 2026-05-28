#![no_std]
use soroban_sdk::{contract, contractimpl, token, Address, Env};

#[contract]
pub struct TransferToSelfVulnerable;

#[contractimpl]
impl TransferToSelfVulnerable {
    // ❌ Transfers directly to the contract itself — tokens permanently locked
    pub fn lock(env: Env, token: Address, from: Address, amount: i128) {
        let client = token::Client::new(&env, &token);
        client.transfer(&from, &env.current_contract_address(), &amount);
    }

    // ❌ Recipient not checked against contract address
    pub fn send(env: Env, token: Address, to: Address, amount: i128) {
        let client = token::Client::new(&env, &token);
        let contract_addr = env.current_contract_address();
        client.transfer(&contract_addr, &to, &amount);
    }
}
