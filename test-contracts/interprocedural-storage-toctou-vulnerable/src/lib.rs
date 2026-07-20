#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct InterproceduralTocTouVulnerable;

const CLAIMED: Symbol = symbol_short!("claimed");

#[contractimpl]
impl InterproceduralTocTouVulnerable {
    fn already_claimed(env: &Env) -> bool {
        env.storage().persistent().has(&CLAIMED)
    }

    fn do_claim(env: &Env) {
        env.storage().persistent().set(&CLAIMED, &true);
    }

    pub fn claim(env: Env) {
        if !Self::already_claimed(&env) {
            Self::do_claim(&env);
        }
    }
}
