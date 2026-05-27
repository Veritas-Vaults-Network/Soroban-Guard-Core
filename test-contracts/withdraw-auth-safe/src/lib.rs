#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct WithdrawAuthSafe;

/// Safe: calls require_auth on the account owner before reading the balance.
#[contractimpl]
impl WithdrawAuthSafe {
    pub fn withdraw(env: Env, from: Address, amount: i128) {
        // Safe: auth is verified before any storage read.
        from.require_auth();
        let balance: i128 = env
            .storage()
            .persistent()
            .get(&from)
            .unwrap_or(0);
        let new_balance = balance - amount;
        env.storage().persistent().set(&from, &new_balance);
    }
}
