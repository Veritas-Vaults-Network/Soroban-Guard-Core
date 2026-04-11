#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct AdminSafe;

#[contractimpl]
impl AdminSafe {
    /// Same entrypoint name with `env.require_auth()` — should pass `unprotected-admin`.
    pub fn set_owner(env: Env, _new_owner: Address) {
        env.require_auth();
    }
}
