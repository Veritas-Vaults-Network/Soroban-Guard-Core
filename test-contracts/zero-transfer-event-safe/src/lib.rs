#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

#[contract]
pub struct ZeroTransferEventSafe;

#[contractimpl]
impl ZeroTransferEventSafe {
    /// ✅ Rejects zero-amount transfers before emitting any event.
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        if amount <= 0 {
            panic!("amount must be positive");
        }
        env.events()
            .publish((symbol_short!("transfer"),), (from.clone(), to.clone(), amount));
        env.storage().instance().set(&to, &amount);
        let _ = from;
    }
}
