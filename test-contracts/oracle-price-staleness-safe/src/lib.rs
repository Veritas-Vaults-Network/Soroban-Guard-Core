#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Env};

const MAX_PRICE_AGE: u64 = 3_600;

#[contracttype]
#[derive(Clone)]
pub struct PriceData {
    pub price: i128,
    pub last_updated: u64,
}

#[contract]
pub struct OraclePriceStalenessSafe;

#[contractimpl]
impl OraclePriceStalenessSafe {
    pub fn set_price(env: Env, price: i128) {
        let data = PriceData {
            price,
            last_updated: env.ledger().timestamp(),
        };
        env.storage().instance().set(&symbol_short!("price"), &data);
    }

    // ✅ `check_price_fresh` is reachable from `swap` (called before the price is used)
    // and compares `last_updated` against the current ledger timestamp.
    pub fn swap(env: Env, amount_in: i128) -> i128 {
        let data: PriceData = env.storage().instance().get(&symbol_short!("price")).unwrap();
        check_price_fresh(&env, &data);
        amount_in * data.price / 1_000_000
    }
}

fn check_price_fresh(env: &Env, data: &PriceData) {
    let now = env.ledger().timestamp();
    assert!(now - data.last_updated <= MAX_PRICE_AGE, "stale oracle price");
}
