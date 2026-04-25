#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct WithdrawAuthVulnerable;

/// Vulnerable: reads balance and writes reduced balance without require_auth.
/// Any caller can drain any account's balance.
#[contractimpl]
impl WithdrawAuthVulnerable {
    pub fn withdraw(env: Env, from: Address, amount: i128) {
        // BUG: no require_auth before reading the balance.
        let balance: i128 = env
            .storage()
            .persistent()
            .get(&from)
            .unwrap_or(0);
        let new_balance = balance - amount;
        env.storage().persistent().set(&from, &new_balance);
    }
}
