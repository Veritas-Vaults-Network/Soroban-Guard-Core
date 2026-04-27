#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol, Val};

#[contract]
pub struct VulnerableContract;

#[contractimpl]
impl VulnerableContract {
    // ❌ invoke_contract result stored directly — malicious callee can inject bad values
    pub fn cache_price(env: Env, oracle: Address) {
        let price: Val =
            env.invoke_contract(&oracle, &Symbol::new(&env, "get_price"), &());
        env.storage()
            .persistent()
            .set(&symbol_short!("price"), &price);
    }
}
