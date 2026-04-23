#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct ContractAddrInLoopVulnerable;

#[contractimpl]
impl ContractAddrInLoopVulnerable {
    pub fn process_items(env: Env, count: u32) {
        // ❌ env.current_contract_address() called inside for loop
        for i in 0..count {
            let addr = env.current_contract_address();
            let _ = (i, addr);
        }
    }

    pub fn process_while(env: Env) {
        let mut i = 0;
        // ❌ env.current_contract_address() called inside while loop
        while i < 10 {
            let addr = env.current_contract_address();
            let _ = addr;
            i += 1;
        }
    }

    pub fn process_loop(env: Env) {
        // ❌ env.current_contract_address() called inside loop
        loop {
            let addr = env.current_contract_address();
            let _ = addr;
            break;
        }
    }
}
