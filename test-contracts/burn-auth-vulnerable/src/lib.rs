#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct BurnAuthVulnerable;

#[contractimpl]
impl BurnAuthVulnerable {
    /// ❌ No require_auth — any caller can destroy tokens belonging to any address.
    pub fn burn(_env: Env, _from: Address, _amount: i128) {}

    /// ❌ No require_auth — any caller can burn on behalf of any spender.
    pub fn burn_from(_env: Env, _spender: Address, _from: Address, _amount: i128) {}
}
