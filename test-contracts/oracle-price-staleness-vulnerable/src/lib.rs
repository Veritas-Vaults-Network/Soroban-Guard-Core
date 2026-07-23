#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Env};

#[contracttype]
#[derive(Clone)]
pub struct PriceData {
    pub price: i128,
    pub last_updated: u64,
}

#[contract]
pub struct OraclePriceStalenessVulnerable;

#[contractimpl]
impl OraclePriceStalenessVulnerable {
    pub fn set_price(env: Env, price: i128) {
        let data = PriceData {
            price,
            last_updated: env.ledger().timestamp(),
        };
        env.storage().instance().set(&symbol_short!("price"), &data);
    }

    // ❌ Reads the oracle price and feeds it straight into a swap calculation. Nothing
    // reachable from `swap` ever compares `last_updated` against the current ledger
    // timestamp, so a stalled oracle feed silently keeps being used.
    pub fn swap(env: Env, amount_in: i128) -> i128 {
        let data: PriceData = env.storage().instance().get(&symbol_short!("price")).unwrap();
        amount_in * data.price / 1_000_000
    }
}
