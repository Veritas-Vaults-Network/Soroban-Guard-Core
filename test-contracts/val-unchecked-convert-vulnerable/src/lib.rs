#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, Val};

#[contract]
pub struct ValUncheckedConvertVulnerable;

#[contractimpl]
impl ValUncheckedConvertVulnerable {
    /// BUG: `.try_into_val().unwrap()` panics if the callee returns an unexpected type.
    pub fn get_balance(env: Env, contract: Address) -> i128 {
        env.invoke_contract::<Val>(
            &contract,
            &Symbol::new(&env, "balance"),
            soroban_sdk::vec![&env],
        )
        .try_into_val(&env)
        .unwrap()
    }

    /// BUG: `.into_val()` directly on a `Val` causes silent type confusion.
    pub fn get_count(env: Env, contract: Address) -> u32 {
        env.invoke_contract::<Val>(
            &contract,
            &Symbol::new(&env, "count"),
            soroban_sdk::vec![&env],
        )
        .into_val(&env)
    }
}
