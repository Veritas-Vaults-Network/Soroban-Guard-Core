#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct UnvalidatedPriceVulnerable;

#[contractimpl]
impl UnvalidatedPriceVulnerable {
    // ❌ No lower or upper bound check — attacker can set price to 0 or i128::MAX
    pub fn set_price(env: Env, price: i128) {
        env.storage().instance().set(&symbol_short!("price"), &price);
    }

    // ❌ Only lower bound — missing upper bound
    pub fn set_rate(env: Env, rate: i128) {
        if rate <= 0 {
            panic!("rate must be positive");
        }
        env.storage().instance().set(&symbol_short!("rate"), &rate);
    }
}
