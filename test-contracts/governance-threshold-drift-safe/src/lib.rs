#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env};

const QUORUM: u32 = 3;

#[contract]
pub struct GovernanceSafe;

#[contractimpl]
impl GovernanceSafe {
    /// A proposal is considered supported when at least QUORUM signers approve.
    pub fn propose(env: Env, proposer: Address, signers: u32) -> bool {
        proposer.require_auth();
        if signers >= QUORUM {
            env.storage()
                .instance()
                .set(&"proposal_active", &true);
            true
        } else {
            false
        }
    }

    /// Execute checks the same QUORUM constant as `propose` — no drift.
    pub fn execute(env: Env, signers: u32) -> bool {
        if signers >= QUORUM {
            env.storage()
                .instance()
                .set(&"executed", &true);
            true
        } else {
            false
        }
    }
}
