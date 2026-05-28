#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Symbol};

#[contract]
pub struct InvokeUncheckedCastVulnerable;

/// Vulnerable: casts invoke_contract result with .unwrap() — a type mismatch
/// silently produces a zero/default value instead of surfacing an error.
#[contractimpl]
impl InvokeUncheckedCastVulnerable {
    pub fn get_balance(env: Env, contract: Address) -> i128 {
        // BUG: .try_into_val(...).unwrap() silently returns 0 on type mismatch.
        env.invoke_contract::<i128>(
            &contract,
            &Symbol::new(&env, "balance"),
            soroban_sdk::vec![&env],
        )
        .try_into_val(&env)
        .unwrap()
    }
}
