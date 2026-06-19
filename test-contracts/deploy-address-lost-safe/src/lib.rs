#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct SafeContract;

#[contractimpl]
impl SafeContract {
    /// Deploys sub-contract and stores address — should pass deploy-address-lost
    pub fn deploy_sub(env: Env) {
        let addr = env.deployer().deploy(
            &env.current_contract_wasm(),
            &[],
            &Symbol::new(&env, "init"),
            &[],
        );
        env.storage()
            .instance()
            .set(&symbol_short!("sub"), &addr);
    }
}
