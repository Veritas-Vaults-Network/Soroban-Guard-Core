#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, Symbol};

#[contract]
pub struct ApproveRaceVulnerable;

#[contractimpl]
impl ApproveRaceVulnerable {
    /// Overwrites allowance directly with no zero reset or transition guard.
    pub fn approve(env: Env, owner: Address, _spender: Address, amount: i128) {
        owner.require_auth();
        let allowance_key = Symbol::new(&env, "allowance");
        env.storage().persistent().set(&allowance_key, &amount);
    }
}
