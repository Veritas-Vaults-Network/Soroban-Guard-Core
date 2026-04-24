#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol};

const ADMIN_KEY: Symbol = symbol_short!("admin");
const BALANCE_KEY: Symbol = symbol_short!("bal");

#[contract]
pub struct MintAuthSafe;

#[contractimpl]
impl MintAuthSafe {
    /// Safe: loads the admin from storage and calls require_auth on that address.
    /// Only the stored admin can authorize a mint.
    pub fn mint(env: Env, to: Address, amount: i128) {
        let admin: Address = env.storage().instance().get(&ADMIN_KEY).unwrap();
        admin.require_auth();
        let current: i128 = env.storage().instance().get(&BALANCE_KEY).unwrap_or(0);
        env.storage().instance().set(&BALANCE_KEY, &(current + amount));
        let _ = to;
    }
}
