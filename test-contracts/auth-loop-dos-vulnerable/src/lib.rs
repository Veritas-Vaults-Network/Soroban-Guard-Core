#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Vec};

#[contract]
pub struct AuthLoopDosVulnerable;

/// Vulnerable: calls require_auth on every element of a storage-backed Vec.
/// An attacker who can grow the signers list can DoS the contract.
#[contractimpl]
impl AuthLoopDosVulnerable {
    pub fn add_signer(env: Env, signer: Address) {
        let mut signers: Vec<Address> = env
            .storage()
            .persistent()
            .get(&symbol_short!("signers"))
            .unwrap_or(soroban_sdk::vec![&env]);
        signers.push_back(signer);
        env.storage()
            .persistent()
            .set(&symbol_short!("signers"), &signers);
    }

    pub fn execute(env: Env) {
        // BUG: require_auth in a loop over a storage-backed list.
        let signers: Vec<Address> = env
            .storage()
            .persistent()
            .get(&symbol_short!("signers"))
            .unwrap();
        for signer in signers {
            signer.require_auth();
        }
    }
}
