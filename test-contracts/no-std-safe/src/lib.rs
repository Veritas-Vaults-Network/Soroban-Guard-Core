#![no_std]
// ✅ Uses only soroban_sdk types — no std:: paths.
use soroban_sdk::{contract, contractimpl, Env, Symbol, Vec};

#[contract]
pub struct GoodContract;

#[contractimpl]
impl GoodContract {
    pub fn hello(env: Env) -> Symbol {
        let _ = Vec::<Symbol>::new(&env);
        Symbol::new(&env, "hello")
    }
}
