#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct AuthTempStorageVulnerable;

#[contractimpl]
impl AuthTempStorageVulnerable {
    /// ❌ Admin read from temporary storage may have expired (TTL elapsed).
    pub fn transfer(env: Env, amount: i128) {
        let admin: Address = env
            .storage()
            .temporary()
            .get(&"admin")
            .unwrap_or_else(|| Address::from_contract_id(&env, &env.current_contract_address()));
        admin.require_auth();
        let _ = amount;
    }
}
