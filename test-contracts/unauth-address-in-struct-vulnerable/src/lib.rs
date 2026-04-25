#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

#[contract]
pub struct UnauthAddressInStructVulnerable;

pub struct Record {
    pub owner: Address,
    pub amount: i128,
}

/// Vulnerable: stores a struct with an Address field from an unauthenticated parameter.
#[contractimpl]
impl UnauthAddressInStructVulnerable {
    pub fn register(env: Env, owner: Address, amount: i128) {
        // BUG: owner is not authenticated — anyone can register any address.
        env.storage()
            .persistent()
            .set(&symbol_short!("rec"), &Record { owner, amount });
    }
}
