#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Symbol};

#[contract]
pub struct UnauthSensitiveReadVulnerable;

#[contractimpl]
impl UnauthSensitiveReadVulnerable {
    /// Returns admin without auth — should trigger `unauth-sensitive-read` (Medium).
    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&Symbol::new(&env, "admin")).unwrap()
    }
}
