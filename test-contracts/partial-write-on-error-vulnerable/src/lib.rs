#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct PartialWriteOnErrorVulnerable;

#[contractimpl]
impl PartialWriteOnErrorVulnerable {
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), ()> {
        // ❌ Write to persistent storage before fallible operation
        // If require_auth() fails, the balance is already updated
        let mut balance: i128 = env.storage().persistent().get(&from).unwrap_or(0);
        balance -= amount;
        env.storage().persistent().set(&from, &balance);

        // Fallible operation - if this fails, state is partially updated
        from.require_auth()?;

        Ok(())
    }

    pub fn withdraw(env: Env, user: Address, amount: i128) -> Result<(), ()> {
        // ❌ Another example: write before return Err
        env.storage().persistent().set(&user, &0i128);
        return Err(()); // If this returns error, state is corrupted
    }
}