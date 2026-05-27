#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct LockContract;

#[contractimpl]
impl LockContract {
    /// ❌ Stores unlock timestamp as u32 — wraps after year 2106, bypassing the lock.
    pub fn lock(env: Env, period: u64) {
        let unlock: u32 = env.ledger().timestamp() + period;
        env.storage().instance().set(&symbol_short!("unlock"), &unlock);
    }
}
