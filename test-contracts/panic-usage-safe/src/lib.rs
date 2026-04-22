#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct PanicUsageSafe;

#[derive(Debug)]
pub enum Error {
    InvalidInput,
    OperationFailed,
}

#[contractimpl]
impl PanicUsageSafe {
    /// Returns Result instead of panicking — safe approach.
    pub fn safe_operation(env: Env) -> Result<(), Error> {
        validate_precondition()?;
        Ok(())
    }

    /// Uses Err return instead of unreachable! — safe approach.
    pub fn validate_path(env: Env, is_valid: bool) -> Result<(), Error> {
        if !is_valid {
            return Err(Error::InvalidInput);
        }
        Ok(())
    }

    /// Handles error cases explicitly without panic.
    pub fn safe_conditional(env: Env, should_fail: bool) -> Result<(), Error> {
        if should_fail {
            return Err(Error::OperationFailed);
        }
        Ok(())
    }
}

fn validate_precondition() -> Result<(), Error> {
    Ok(())
}
