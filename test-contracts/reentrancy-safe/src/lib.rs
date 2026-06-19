#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

#[contract]
pub struct SafeContract;

const KEY: u32 = 0;

#[contractimpl]
impl SafeContract {
    pub fn transfer(env: Env, to: Address, amount: i128) {
        // ✅ Storage write committed BEFORE external call
        env.storage().persistent().set(&KEY, &amount);
        env.invoke_contract::<()>(
            &to,
            &symbol_short!("callback"),
            &soroban_sdk::vec![&env],
        );
    }
}
