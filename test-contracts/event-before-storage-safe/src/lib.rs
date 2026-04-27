#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct EventBeforeStorageSafe;

const BAL: Symbol = symbol_short!("bal");

#[contractimpl]
impl EventBeforeStorageSafe {
    /// ✅ Storage write completes first; event is only emitted after state is committed.
    pub fn deposit(env: Env, amount: i128) {
        env.storage().persistent().set(&BAL, &amount);
        env.events().publish((symbol_short!("deposit"),), amount);
    }
}
