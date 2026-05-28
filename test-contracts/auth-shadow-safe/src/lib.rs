#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol};

#[contract]
pub struct AuthShadowSafe;

const ADMIN: Symbol = symbol_short!("admin");

#[contractimpl]
impl AuthShadowSafe {
    /// ✅ Uses env.require_auth() instead of parameter.require_auth().
    pub fn transfer(env: Env, admin: Address, recipient: Address, amount: i128) {
        env.require_auth();
        let stored_admin: Address = env
            .storage()
            .persistent()
            .get(&ADMIN)
            .unwrap_or_else(|| Address::from_contract_id(&env, &env.current_contract_address()));
        let _ = (admin, recipient, amount, stored_admin);
    }

    /// ✅ Uses different parameter name to avoid shadowing.
    pub fn withdraw(env: Env, caller: Address, amount: i128) {
        caller.require_auth();
        let stored_admin: Address = env
            .storage()
            .persistent()
            .get(&ADMIN)
            .unwrap_or_else(|| Address::from_contract_id(&env, &env.current_contract_address()));
        let _ = (amount, stored_admin);
    }
}
