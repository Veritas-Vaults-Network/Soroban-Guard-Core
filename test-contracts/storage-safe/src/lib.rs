#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct StorageSafe;

const KEY: Symbol = symbol_short!("data");

#[contractimpl]
impl StorageSafe {
    /// Persistent storage + fixed `symbol_short!` key — should pass `unsafe-storage-patterns`.
    pub fn put(env: Env, v: u32) {
        env.require_auth();
        env.storage().persistent().set(&KEY, &v);
    }
}
