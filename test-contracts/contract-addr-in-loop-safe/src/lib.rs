#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct ContractAddrInLoopSafe;

#[contractimpl]
impl ContractAddrInLoopSafe {
    pub fn process_items(env: Env, count: u32) {
        // ✅ Cache contract address before the loop
        let addr = env.current_contract_address();
        for i in 0..count {
            let _ = (i, &addr);
        }
    }

    pub fn process_while(env: Env) {
        // ✅ Cache contract address before the loop
        let addr = env.current_contract_address();
        let mut i = 0;
        while i < 10 {
            let _ = &addr;
            i += 1;
        }
    }

    pub fn process_outside_loop(env: Env) {
        // ✅ Call outside of any loop
        let addr = env.current_contract_address();
        let _ = addr;
    }
}
