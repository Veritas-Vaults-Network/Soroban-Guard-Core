#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

#[contract]
pub struct AddrParamNoAuthVulnerable;

#[contractimpl]
impl AddrParamNoAuthVulnerable {
    /// BUG: `user` is an Address parameter but require_auth is never called on it.
    /// Any caller can write to storage on behalf of any address.
    pub fn deposit(env: Env, user: Address, amount: i128) {
        let _ = user; // address accepted but never authenticated
        env.storage()
            .persistent()
            .set(&symbol_short!("bal"), &amount);
    }
}
