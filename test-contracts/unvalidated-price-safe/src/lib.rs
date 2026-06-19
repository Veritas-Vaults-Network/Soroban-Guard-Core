#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

const MIN_PRICE: i128 = 1;
const MAX_PRICE: i128 = 1_000_000_000;

#[contract]
pub struct UnvalidatedPriceSafe;

#[contractimpl]
impl UnvalidatedPriceSafe {
    // ✅ Both lower and upper bounds validated
    pub fn set_price(env: Env, price: i128) {
        assert!(price >= MIN_PRICE && price <= MAX_PRICE);
        env.storage().instance().set(&symbol_short!("price"), &price);
    }

    // ✅ Both bounds validated
    pub fn set_rate(env: Env, rate: i128) {
        assert!(rate > 0 && rate <= 10_000);
        env.storage().instance().set(&symbol_short!("rate"), &rate);
    }
}
