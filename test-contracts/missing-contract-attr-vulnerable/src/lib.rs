#![no_std]
use soroban_sdk::{contractimpl, Env};

/// Missing #[contract] attribute — should trigger missing-contract-attr
pub struct VulnerableContract;

#[contractimpl]
impl VulnerableContract {
    pub fn test(env: Env) {}
}
