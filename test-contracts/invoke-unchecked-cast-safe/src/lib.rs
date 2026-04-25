#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Symbol};

#[contract]
pub struct InvokeUncheckedCastSafe;

/// Safe: uses match to handle the cast result explicitly.
#[contractimpl]
impl InvokeUncheckedCastSafe {
    pub fn get_balance(env: Env, contract: Address) -> Option<i128> {
        let result = env.invoke_contract::<i128>(
            &contract,
            &Symbol::new(&env, "balance"),
            soroban_sdk::vec![&env],
        );
        // Safe: type error is handled explicitly rather than silently swallowed.
        match result.try_into_val(&env) {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    }
}
