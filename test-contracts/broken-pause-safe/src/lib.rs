#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct BrokenPauseSafe;

/// Safe: pause/unpause write and clear the paused flag in storage.
#[contractimpl]
impl BrokenPauseSafe {
    pub fn pause(env: Env) {
        env.require_auth();
        env.storage().instance().set(&symbol_short!("paused"), &true);
    }

    pub fn unpause(env: Env) {
        env.require_auth();
        env.storage().instance().set(&symbol_short!("paused"), &false);
    }
}
