#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

#[contract]
pub struct ScaleFactorDriftSafe;

#[contractimpl]
impl ScaleFactorDriftSafe {
    /// Stores the balance scaled to 7-decimal stroops.
    pub fn deposit(env: Env, user: Address, amount: i128) {
        user.require_auth();
        let key = (symbol_short!("bal"), user);
        let scaled = amount * 10_000_000;
        env.storage().persistent().set(&key, &scaled);
    }

    /// Reads the balance back using the same 7-decimal scale factor as `deposit`.
    pub fn withdraw(env: Env, user: Address) -> i128 {
        user.require_auth();
        let key = (symbol_short!("bal"), user);
        let raw: i128 = env.storage().persistent().get(&key).unwrap();
        raw / 10_000_000
    }
}
