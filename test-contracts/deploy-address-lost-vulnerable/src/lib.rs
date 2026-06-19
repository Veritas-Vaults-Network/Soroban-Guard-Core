#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct VulnerableContract;

#[contractimpl]
impl VulnerableContract {
    /// Deploys sub-contract but doesn't store address — should trigger deploy-address-lost
    pub fn deploy_sub(env: Env) {
        env.deployer().deploy(
            &env.current_contract_wasm(),
            &[],
            &soroban_sdk::Symbol::new(&env, "init"),
            &[],
        );
    }
}
