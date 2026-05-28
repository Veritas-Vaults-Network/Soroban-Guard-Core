#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol};

#[contract]
pub struct RedundantAuthArgsVulnerable;

const ADMIN: Symbol = symbol_short!("admin");

#[contractimpl]
impl RedundantAuthArgsVulnerable {
    /// ❌ All args are string literals — defeats the purpose of require_auth_for_args.
    pub fn bad_auth(env: Env) {
        env.require_auth_for_args(("literal_arg", "another_literal"));
    }

    /// ❌ Args are all read from storage — not call-specific.
    pub fn bad_auth_storage(env: Env) {
        let admin: Address = env
            .storage()
            .persistent()
            .get(&ADMIN)
            .unwrap_or_else(|| Address::from_contract_id(&env, &env.current_contract_address()));
        env.require_auth_for_args((admin,));
    }
}
