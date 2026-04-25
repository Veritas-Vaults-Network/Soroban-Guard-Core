#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct DebugEntrypointVulnerable;

/// Vulnerable: debug/dev entrypoints left in production — world-callable backdoors.
#[contractimpl]
impl DebugEntrypointVulnerable {
    pub fn dev_mint(env: Env, to: Address, amount: i128) {
        // BUG: no auth — anyone can mint tokens.
        let _ = (env, to, amount);
    }

    pub fn debug_state(env: Env) -> u32 {
        let _ = env;
        42
    }
}
