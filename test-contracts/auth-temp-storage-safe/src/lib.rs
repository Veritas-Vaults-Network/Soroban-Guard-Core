#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct AuthTempStorageSafe;

#[contractimpl]
impl AuthTempStorageSafe {
    /// ✅ Admin read from persistent storage (survives TTL).
    pub fn transfer(env: Env, amount: i128) {
        let admin: Address = env
            .storage()
            .persistent()
            .get(&"admin")
            .unwrap_or_else(|| Address::from_contract_id(&env, &env.current_contract_address()));
        admin.require_auth();
        let _ = amount;
    }

    /// ✅ Uses env.require_auth() instead of reading from storage.
    pub fn withdraw(env: Env, amount: i128) {
        env.require_auth();
        let _ = amount;
    }
}
