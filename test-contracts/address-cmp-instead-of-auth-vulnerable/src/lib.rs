#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct AddressCmpInsteadOfAuthVulnerable;

#[contractimpl]
impl AddressCmpInsteadOfAuthVulnerable {
    /// ❌ Compares caller with admin using == instead of require_auth.
    /// Bypasses Soroban's host-level signature verification.
    pub fn transfer(env: Env, caller: Address, amount: i128) {
        let admin: Address = env
            .storage()
            .persistent()
            .get(&"admin")
            .unwrap_or_else(|| Address::from_contract_id(&env, &env.current_contract_address()));
        if caller == admin {
            let _ = amount;
        }
    }
}
