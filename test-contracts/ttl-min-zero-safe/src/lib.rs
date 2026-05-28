#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct TtlMinZeroSafe;

const BALANCE: Symbol = symbol_short!("balance");
const CONFIG: Symbol = symbol_short!("config");

#[contractimpl]
impl TtlMinZeroSafe {
    pub fn set_balance(env: Env, amount: i128) {
        env.storage().persistent().set(&BALANCE, &amount);
        // ✅ min = 5000: the entry is refreshed to 10000 whenever it drops
        //    below 5000 ledgers remaining — guarantees a meaningful TTL floor.
        env.storage().persistent().extend_ttl(&BALANCE, 5000, 10000);
    }

    pub fn refresh_instance(env: Env) {
        // ✅ min is half of max — entry is extended well before expiry.
        env.storage().instance().extend_ttl(2500, 5000);
    }

    pub fn init(env: Env, val: u32) {
        env.storage().persistent().set(&CONFIG, &val);
        // ✅ Sensible min threshold (roughly half of max).
        env.storage().persistent().extend_ttl(&CONFIG, 8640, 17280);
    }
}
