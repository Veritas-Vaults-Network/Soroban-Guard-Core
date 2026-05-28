#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, Val};

#[contract]
pub struct ValUncheckedConvertSafe;

#[contractimpl]
impl ValUncheckedConvertSafe {
    /// ✅ Handle the conversion Result explicitly with match.
    pub fn get_balance(env: Env, contract: Address) -> Option<i128> {
        let val: Val = env.invoke_contract(
            &contract,
            &Symbol::new(&env, "balance"),
            soroban_sdk::vec![&env],
        );
        match val.try_into_val(&env) {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    }

    /// ✅ Use try_into_val and propagate the error with ?.
    pub fn get_count(env: Env, contract: Address) -> Result<u32, soroban_sdk::Error> {
        let val: Val = env.invoke_contract(
            &contract,
            &Symbol::new(&env, "count"),
            soroban_sdk::vec![&env],
        );
        let count: u32 = val.try_into_val(&env)?;
        Ok(count)
    }
}
