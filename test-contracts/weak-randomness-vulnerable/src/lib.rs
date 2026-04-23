#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct LotteryContract;

#[contractimpl]
impl LotteryContract {
    // ❌ Uses ledger timestamp as randomness — validators can manipulate this.
    pub fn roll(env: Env) -> u64 {
        env.ledger().timestamp() % 6 + 1
    }
}
