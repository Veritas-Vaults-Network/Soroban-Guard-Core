#![no_std]
use soroban_sdk::{contract, contractimpl, Env, InvokeContractArgs};

#[contract]
pub struct AuthorizeEmptySafe;

#[contractimpl]
impl AuthorizeEmptySafe {
    /// ✅ Authorizes specific sub-contract invocations.
    pub fn good_authorize(env: Env, args: InvokeContractArgs) {
        env.authorize_as_current_contract(&[args]);
    }

    /// ✅ No authorize call (not needed for this operation).
    pub fn simple_operation(env: Env) {
        let _ = env;
    }
}
