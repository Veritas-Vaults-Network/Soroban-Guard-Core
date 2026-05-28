#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

#[contract]
pub struct AuthLoopDosSafe;

/// Safe: uses a single designated admin rather than iterating a list for auth.
#[contractimpl]
impl AuthLoopDosSafe {
    pub fn execute(env: Env) {
        // Safe: single require_auth on a stored admin address.
        let admin: Address = env
            .storage()
            .persistent()
            .get(&symbol_short!("admin"))
            .unwrap();
        admin.require_auth();
    }
}
