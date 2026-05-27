#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

/// Has #[contract] attribute — should pass missing-contract-attr
#[contract]
pub struct SafeContract;

#[contractimpl]
impl SafeContract {
    pub fn test(env: Env) {}
}
