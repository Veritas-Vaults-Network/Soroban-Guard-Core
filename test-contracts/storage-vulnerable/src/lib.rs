#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct StorageVulnerable;

const KEY: Symbol = symbol_short!("cache");

#[contractimpl]
impl StorageVulnerable {
    /// Writes to temporary storage (TTL-bound) — should trigger `unsafe-storage-patterns` (Medium).
    pub fn stash(env: Env, v: u32) {
        env.require_auth();
        env.storage().temporary().set(&KEY, &v);
    }
}
