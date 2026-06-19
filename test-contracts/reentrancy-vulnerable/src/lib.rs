#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

#[contract]
pub struct VulnerableContract;

const KEY: u32 = 0;

#[contractimpl]
impl VulnerableContract {
    pub fn transfer(env: Env, to: Address, amount: i128) {
        // ❌ External call happens BEFORE storage write — reentrancy risk
        env.invoke_contract::<()>(
            &to,
            &symbol_short!("callback"),
            &soroban_sdk::vec![&env],
        );
        env.storage().persistent().set(&KEY, &amount);
    }
}
