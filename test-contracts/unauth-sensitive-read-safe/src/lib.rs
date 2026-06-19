#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Symbol};

#[contract]
pub struct UnauthSensitiveReadSafe;

#[contractimpl]
impl UnauthSensitiveReadSafe {
    /// Returns admin with auth — should pass `unauth-sensitive-read`.
    pub fn get_admin(env: Env) -> Address {
        env.require_auth();
        env.storage().instance().get(&Symbol::new(&env, "admin")).unwrap()
    }
}
