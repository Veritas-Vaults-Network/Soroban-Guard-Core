#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

#[contract]
pub struct UnauthAddressInStructSafe;

pub struct Record {
    pub owner: Address,
    pub amount: i128,
}

/// Safe: calls require_auth on the owner address before storing the struct.
#[contractimpl]
impl UnauthAddressInStructSafe {
    pub fn register(env: Env, owner: Address, amount: i128) {
        // Safe: owner must prove they control the address.
        owner.require_auth();
        env.storage()
            .persistent()
            .set(&symbol_short!("rec"), &Record { owner, amount });
    }
}
