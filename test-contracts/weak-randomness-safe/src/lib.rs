#![no_std]
use soroban_sdk::{contract, contractimpl, Bytes, Env};

#[contract]
pub struct LotteryContract;

#[contractimpl]
impl LotteryContract {
    // ✅ Uses a commit-reveal seed supplied by the caller — not ledger metadata.
    pub fn roll(_env: Env, seed: Bytes) -> u64 {
        // In production: verify the seed against a prior commitment.
        let first_byte = seed.get(0).unwrap_or(0) as u64;
        first_byte % 6 + 1
    }

    // ✅ Ledger timestamp used only for comparison (deadline check), not as randomness.
    pub fn is_expired(env: Env, deadline: u64) -> bool {
        env.ledger().timestamp() > deadline
    }
}
