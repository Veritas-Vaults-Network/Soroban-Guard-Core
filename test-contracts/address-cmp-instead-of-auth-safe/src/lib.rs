#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct AddressCmpInsteadOfAuthSafe;

#[contractimpl]
impl AddressCmpInsteadOfAuthSafe {
    /// ✅ Uses require_auth() for proper signature verification.
    pub fn transfer(env: Env, amount: i128) {
        env.require_auth();
        let _ = amount;
    }

    /// ✅ Reads admin and calls require_auth on it.
    pub fn withdraw(env: Env, amount: i128) {
        let admin: Address = env
            .storage()
            .persistent()
            .get(&"admin")
            .unwrap_or_else(|| Address::from_contract_id(&env, &env.current_contract_address()));
        admin.require_auth();
        let _ = amount;
    }
}
