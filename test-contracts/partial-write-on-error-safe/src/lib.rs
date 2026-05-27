#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct PartialWriteOnErrorSafe;

#[contractimpl]
impl PartialWriteOnErrorSafe {
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), ()> {
        // ✅ Check authorization first
        from.require_auth()?;

        // ✅ Then update storage
        let mut balance: i128 = env.storage().persistent().get(&from).unwrap_or(0);
        balance -= amount;
        env.storage().persistent().set(&from, &balance);

        Ok(())
    }

    pub fn withdraw(env: Env, user: Address, amount: i128) -> Result<(), ()> {
        // ✅ Validate first
        let balance: i128 = env.storage().persistent().get(&user).unwrap_or(0);
        if balance < amount {
            return Err(());
        }

        // ✅ Then update
        user.require_auth()?;
        env.storage().persistent().set(&user, &(balance - amount));

        Ok(())
    }
}