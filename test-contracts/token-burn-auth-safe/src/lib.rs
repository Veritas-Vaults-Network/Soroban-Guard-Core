#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct TokenBurnAuthSafe;

#[contractimpl]
impl TokenBurnAuthSafe {
    pub fn burn_tokens(env: Env, token: Address, from: Address, amount: i128) {
        // ✅ Require auth on the from address before burning
        from.require_auth();
        let token_client = soroban_sdk::token::Client::new(&env, &token);
        token_client.burn(&from, &amount);
    }

    pub fn burn_with_args_auth(env: Env, token: Address, from: Address, amount: i128) {
        // ✅ Use require_auth_for_args to authorize all parameters
        env.require_auth_for_args((token, from, amount));
        let token_client = soroban_sdk::token::Client::new(&env, &token);
        token_client.burn(&from, &amount);
    }

    pub fn burn_from_invoker(env: Env, token: Address, amount: i128) {
        let from = env.invoker();
        // ✅ Even with invoker, require_auth is needed
        from.require_auth();
        let token_client = soroban_sdk::token::Client::new(&env, &token);
        token_client.burn(&from, &amount);
    }
}