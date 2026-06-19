#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct DebugEntrypointSafe;

/// Safe: no debug/test/dev entrypoints in production code.
#[contractimpl]
impl DebugEntrypointSafe {
    pub fn mint(env: Env, to: Address, amount: i128) {
        env.require_auth();
        let _ = (to, amount);
    }
}
