#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

#[contract]
pub struct UnauthAddressTupleSafe;

#[contractimpl]
impl UnauthAddressTupleSafe {
    /// Safe: `from` must prove they control the address before the pair is stored.
    pub fn approve(env: Env, from: Address, to: Address) {
        from.require_auth();
        env.storage()
            .persistent()
            .set(&symbol_short!("appr"), &(from, to));
    }
}
