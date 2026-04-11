#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct AdminVulnerable;

#[contractimpl]
impl AdminVulnerable {
    /// Privileged name, no auth call — should trigger `unprotected-admin` (High).
    pub fn set_owner(_env: Env, _new_owner: Address) {}
}
