#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

#[contract]
pub struct AddrParamNoAuthSafe;

#[contractimpl]
impl AddrParamNoAuthSafe {
    /// Safe: `user` must prove they control the address before the storage write.
    pub fn deposit(env: Env, user: Address, amount: i128) {
        user.require_auth();
        env.storage()
            .persistent()
            .set(&symbol_short!("bal"), &amount);
    }
}
