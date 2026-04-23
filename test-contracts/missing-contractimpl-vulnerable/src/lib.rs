#![no_std]
use soroban_sdk::{contract, Env};

#[contract]
pub struct MissingContractimplVulnerable;

/// ❌ Missing #[contractimpl] — these pub methods are NOT exposed as contract entrypoints.
impl MissingContractimplVulnerable {
    pub fn hello(_env: Env) -> u32 {
        42
    }
}
