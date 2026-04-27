#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

#[contract]
pub struct UnauthAddressTupleVulnerable;

#[contractimpl]
impl UnauthAddressTupleVulnerable {
    /// BUG: stores (from, to) tuple without calling require_auth on either address.
    /// An attacker can record arbitrary address pairs as fake approvals.
    pub fn approve(env: Env, from: Address, to: Address) {
        env.storage()
            .persistent()
            .set(&symbol_short!("appr"), &(from, to));
    }
}
