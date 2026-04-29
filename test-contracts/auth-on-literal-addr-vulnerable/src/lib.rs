#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct VulnerableContract;

#[contractimpl]
impl VulnerableContract {
    /// ❌ require_auth() is called on the contract's own address — not the caller.
    /// This provides no real access control.
    pub fn admin_action(env: Env) {
        let contract = env.current_contract_address();
        contract.require_auth();
        // ... privileged logic
    }
}
