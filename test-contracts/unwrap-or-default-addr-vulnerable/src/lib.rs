#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    // ❌ unwrap_or_default() on Option<Address> yields the zero-address,
    //    which is not a valid Stellar account.
    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get::<_, Address>(&symbol_short!("admin"))
            .unwrap_or_default()
    }

    pub fn get_recipient(env: Env) -> Address {
        let recipient: Address = env
            .storage()
            .persistent()
            .get(&symbol_short!("recip"))
            .unwrap_or_default();
        recipient
    }
}
