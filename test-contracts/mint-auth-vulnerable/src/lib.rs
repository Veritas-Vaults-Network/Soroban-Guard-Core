#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol};

const BALANCE_KEY: Symbol = symbol_short!("bal");

#[contract]
pub struct MintAuthVulnerable;

#[contractimpl]
impl MintAuthVulnerable {
    /// Vulnerable: calls require_auth on the recipient `to` instead of an admin.
    /// Any user can sign their own transaction and mint tokens to themselves.
    pub fn mint(env: Env, to: Address, amount: i128) {
        to.require_auth();
        let current: i128 = env.storage().instance().get(&BALANCE_KEY).unwrap_or(0);
        env.storage().instance().set(&BALANCE_KEY, &(current + amount));
    }
}
