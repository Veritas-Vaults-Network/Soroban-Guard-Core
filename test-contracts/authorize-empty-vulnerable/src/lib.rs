#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct AuthorizeEmptyVulnerable;

#[contractimpl]
impl AuthorizeEmptyVulnerable {
    /// ❌ Authorizes nothing but still consumes compute.
    /// Likely a bug — developer intended to authorize specific sub-contract calls.
    pub fn bad_authorize(env: Env) {
        env.authorize_as_current_contract(&[]);
    }
}
