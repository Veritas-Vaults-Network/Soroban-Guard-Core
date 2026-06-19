#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct SequenceAsKeySafe;

const KEY: soroban_sdk::Symbol = symbol_short!("data");

#[contractimpl]
impl SequenceAsKeySafe {
    pub fn store_safely(env: Env, value: u32) {
        env.storage().persistent().set(&KEY, &value);
    }

    pub fn store_with_sequence_in_temp(env: Env, value: u32) {
        let seq = env.ledger().sequence();
        env.storage().temporary().set(&seq, &value);
    }
}
