#![no_std]
use soroban_sdk::{contract, contractimpl, token, Address, Env, Vec};

#[contract]
pub struct UnboundedBatchSafe;

const MAX_RECIPIENTS: u32 = 100;

#[contractimpl]
impl UnboundedBatchSafe {
    // ✅ .len() guard prevents compute DoS
    pub fn distribute(env: Env, token: Address, recipients: Vec<Address>, amount: i128) {
        assert!(recipients.len() <= MAX_RECIPIENTS);
        let client = token::Client::new(&env, &token);
        let from = env.current_contract_address();
        for recipient in recipients.iter() {
            client.transfer(&from, &recipient, &amount);
        }
    }
}
