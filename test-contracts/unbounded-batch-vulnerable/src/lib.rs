#![no_std]
use soroban_sdk::{contract, contractimpl, token, Address, Env, Vec};

#[contract]
pub struct UnboundedBatchVulnerable;

#[contractimpl]
impl UnboundedBatchVulnerable {
    // ❌ No .len() guard — attacker can pass huge Vec to exhaust compute budget
    pub fn distribute(env: Env, token: Address, recipients: Vec<Address>, amount: i128) {
        let client = token::Client::new(&env, &token);
        let from = env.current_contract_address();
        for recipient in recipients.iter() {
            client.transfer(&from, &recipient, &amount);
        }
    }
}
