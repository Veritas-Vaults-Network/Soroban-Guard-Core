#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct AuthorizeAsContractVulnerable;

#[contractimpl]
impl AuthorizeAsContractVulnerable {
    /// Calls authorize_as_current_contract without prior require_auth.
    pub fn vulnerable_authorize(env: Env) {
        env.authorize_as_current_contract();
    }

    /// Also vulnerable when only authorize_as_current_contract is used.
    pub fn vulnerable_authorize_with_args(env: Env, admin: Address) {
        env.authorize_as_current_contract();
        let _ = admin;
    }
}