#![no_std]

use soroban_sdk::{contract, contractimpl, token, Address, Env};

#[contract]
pub struct CrossTokenProvenanceMixVulnerable;

#[contractimpl]
impl CrossTokenProvenanceMixVulnerable {
    pub fn swap(env: Env, token_a: Address, token_b: Address, amount_a: i128, amount_b: i128, to: Address) {
        // `amount_a` and `amount_b` are denominated in `token_a` and `token_b`
        // respectively - adding them together mixes units with no conversion.
        let total = amount_a + amount_b;
        let client = token::Client::new(&env, &token_a);
        client.transfer(&env.current_contract_address(), &to, &total);
        let _ = token_b;
    }
}
