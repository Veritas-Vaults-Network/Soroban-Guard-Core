#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct MissingContractimplSafe;

/// ✅ #[contractimpl] present — methods are properly exposed as contract entrypoints.
#[contractimpl]
impl MissingContractimplSafe {
    pub fn hello(_env: Env) -> u32 {
        42
    }
}
