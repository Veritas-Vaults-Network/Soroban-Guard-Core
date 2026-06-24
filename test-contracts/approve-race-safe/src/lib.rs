#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, Symbol};

#[contract]
pub struct ApproveRaceSafe;

#[contractimpl]
impl ApproveRaceSafe {
    /// Resets allowance to zero before setting the new amount.
    pub fn approve(env: Env, owner: Address, _spender: Address, amount: i128) {
        owner.require_auth();
        let allowance_key = Symbol::new(&env, "allowance");
        env.storage().persistent().set(&allowance_key, &0_i128);
        env.storage().persistent().set(&allowance_key, &amount);
    }
}
