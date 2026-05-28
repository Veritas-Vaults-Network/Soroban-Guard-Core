#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct TokenBurnAuthVulnerable;

#[contractimpl]
impl TokenBurnAuthVulnerable {
    pub fn burn_tokens(env: Env, token: Address, from: Address, amount: i128) {
        let token_client = soroban_sdk::token::Client::new(&env, &token);
        // ❌ Burn without require_auth on from address
        // Anyone can call this to burn tokens from any address
        token_client.burn(&from, &amount);
    }

    pub fn burn_from_sender(env: Env, token: Address, amount: i128) {
        let from = env.invoker();
        let token_client = soroban_sdk::token::Client::new(&env, &token);
        // ❌ Even when using env.invoker(), still need require_auth
        token_client.burn(&from, &amount);
    }
}