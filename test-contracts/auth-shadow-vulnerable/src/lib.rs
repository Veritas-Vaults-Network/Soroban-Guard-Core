#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol};

#[contract]
pub struct AuthShadowVulnerable;

const ADMIN: Symbol = symbol_short!("admin");

#[contractimpl]
impl AuthShadowVulnerable {
    /// ❌ Parameter `admin` shadows storage key "admin". Calling require_auth(&admin)
    /// authenticates the parameter, not the stored admin.
    pub fn transfer(env: Env, admin: Address, recipient: Address, amount: i128) {
        admin.require_auth();
        let stored_admin: Address = env
            .storage()
            .persistent()
            .get(&ADMIN)
            .unwrap_or_else(|| Address::from_contract_id(&env, &env.current_contract_address()));
        let _ = (recipient, amount, stored_admin);
    }
}
