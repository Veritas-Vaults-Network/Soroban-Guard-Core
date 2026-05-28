#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

#[contract]
pub struct ZeroTransferEventVulnerable;

#[contractimpl]
impl ZeroTransferEventVulnerable {
    /// ❌ Emits a transfer event without checking amount > 0.
    /// Zero-amount transfers spam the event log with meaningless entries.
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        // No zero-amount guard — event is emitted even for amount == 0
        env.events()
            .publish((symbol_short!("transfer"),), (from, to, amount));
        env.storage().instance().set(&to, &amount);
    }
}
