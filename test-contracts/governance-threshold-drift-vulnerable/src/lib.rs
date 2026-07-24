#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct GovernanceVulnerable;

#[contractimpl]
impl GovernanceVulnerable {
    /// A proposal is considered supported when at least 3 signers approve.
    pub fn propose(env: Env, proposer: Address, signers: u32) -> bool {
        proposer.require_auth();
        if signers >= 3 {
            env.storage()
                .instance()
                .set(&"proposal_active", &true);
            true
        } else {
            false
        }
    }

    /// Execute checks that the proposal has enough support — but uses a
    /// different (lower) threshold than `propose`, which is a governance-bypass
    /// bug: a proposal that failed the propose gate can still be executed.
    pub fn execute(env: Env, signers: u32) -> bool {
        if signers >= 2 {
            env.storage()
                .instance()
                .set(&"executed", &true);
            true
        } else {
            false
        }
    }
}
