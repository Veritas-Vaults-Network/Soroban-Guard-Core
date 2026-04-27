#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol};

#[contract]
pub struct SafeContract;

#[contractimpl]
impl SafeContract {
    // ✅ Validate the returned value before storing it
    pub fn cache_price(env: Env, oracle: Address) {
        let price: i128 =
            env.invoke_contract(&oracle, &Symbol::new(&env, "get_price"), &());
        // Validate range before persisting
        assert!(price > 0, "price must be positive");
        let validated: i128 = price;
        env.storage()
            .persistent()
            .set(&symbol_short!("price"), &validated);
    }
}
