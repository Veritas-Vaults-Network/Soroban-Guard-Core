#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct AuthorizeAsContractSafe;

#[contractimpl]
impl AuthorizeAsContractSafe {
    /// Uses require_auth before authorize_as_current_contract.
    pub fn safe_authorize(env: Env, admin: Address) {
        env.require_auth(&admin);
        env.authorize_as_current_contract();
    }

    /// Uses require_auth_for_args before authorize_as_current_contract.
    pub fn safe_authorize_args(env: Env, admin: Address) {
        env.require_auth_for_args((admin, 123));
        env.authorize_as_current_contract();
    }

    /// No authorization escalation needed.
    pub fn no_authorize(env: Env) {
        // Safe: no authorize_as_current_contract call.
    }
}