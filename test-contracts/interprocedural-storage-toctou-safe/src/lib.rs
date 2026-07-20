#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct InterproceduralTocTouSafe;

const CLAIMED: Symbol = symbol_short!("claimed");

#[contractimpl]
impl InterproceduralTocTouSafe {
    fn already_claimed_and_set(env: &Env) -> bool {
        if env.storage().persistent().has(&CLAIMED) {
            return true;
        }
        env.storage().persistent().set(&CLAIMED, &true);
        false
    }

    pub fn claim(env: Env) {
        Self::already_claimed_and_set(&env);
    }
}
