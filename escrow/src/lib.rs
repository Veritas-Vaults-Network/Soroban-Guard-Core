//! LiquiFact Escrow Contract
//!
//! # State machine
//!
//! ```text
//! 0 (Funding) ──admin cancel_funding──► 4 (Cancelled)
//!      │                                      │
//!      │ (target reached)              investor refund
//!      ▼                                      ▼
//!   1 (Active)                         contribution → 0
//! ```
//!
//! Status codes:
//! - `0` — Funding: accepting contributions, target not yet reached
//! - `1` — Active: funding target met, escrow is live
//! - `2` — Completed: escrow obligations fulfilled
//! - `3` — Legal hold: frozen by compliance action
//! - `4` — Cancelled: funding failed; investors may claim refunds

#![no_std]

use soroban_sdk::{contract, contractevent, contractimpl, contracttype, token, Address, Env};

mod external_calls;

#[cfg(test)]
mod tests {
    mod funding;
}

// ── Status constants ──────────────────────────────────────────────────────────

const STATUS_FUNDING: u32 = 0;
const STATUS_CANCELLED: u32 = 4;

// ── Storage keys ─────────────────────────────────────────────────────────────

/// Persistent storage keys for the escrow contract.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Address of the contract administrator.
    Admin,
    /// Address of the SEP-41 funding token.
    FundingToken,
    /// Funding target (in token stroops).
    FundingTarget,
    /// Total amount funded so far.
    FundedAmount,
    /// Current lifecycle status (u32).
    Status,
    /// Legal hold flag (bool).
    LegalHold,
    /// Per-investor contribution amount.
    InvestorContribution(Address),
}

// ── Events ────────────────────────────────────────────────────────────────────

/// Emitted when an admin cancels a funding-phase escrow.
#[contractevent]
pub struct FundingCancelled {
    pub admin: Address,
    pub funded_amount: i128,
}

/// Emitted when an investor successfully claims a refund.
#[contractevent]
pub struct Refunded {
    pub investor: Address,
    pub amount: i128,
}

// ── Contract ──────────────────────────────────────────────────────────────────

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    // ── Initialisation ────────────────────────────────────────────────────────

    /// Initialise the escrow.
    ///
    /// Must be called once by the deployer. Sets status to `0` (Funding).
    ///
    /// # Arguments
    /// * `admin`          – Address that may call `cancel_funding`.
    /// * `funding_token`  – SEP-41 token contract address.
    /// * `funding_target` – Minimum amount (in stroops) to reach Active status.
    pub fn initialize(
        env: Env,
        admin: Address,
        funding_token: Address,
        funding_target: i128,
    ) {
        // Prevent re-initialisation.
        if env.storage().instance().has(&DataKey::Status) {
            panic!("already initialised");
        }
        assert!(funding_target > 0, "funding_target must be positive");

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::FundingToken, &funding_token);
        env.storage()
            .instance()
            .set(&DataKey::FundingTarget, &funding_target);
        env.storage()
            .instance()
            .set(&DataKey::FundedAmount, &0_i128);
        env.storage()
            .instance()
            .set(&DataKey::Status, &STATUS_FUNDING);
        env.storage()
            .instance()
            .set(&DataKey::LegalHold, &false);

        env.storage()
            .instance()
            .extend_ttl(17_280, 17_280 * 30);
    }

    // ── Admin: cancel funding ─────────────────────────────────────────────────

    /// Transition the escrow from `Funding` (0) to `Cancelled` (4).
    ///
    /// # Access control
    /// Requires `admin` authorisation.
    ///
    /// # Preconditions
    /// - Status must be `0` (Funding).
    /// - Legal hold must not be active.
    ///
    /// # Effects
    /// - Sets status to `4` (Cancelled).
    /// - Emits [`FundingCancelled`].
    pub fn cancel_funding(env: Env) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("not initialised");
        admin.require_auth();

        let legal_hold: bool = env
            .storage()
            .instance()
            .get(&DataKey::LegalHold)
            .unwrap_or(false);
        assert!(!legal_hold, "escrow is under legal hold");

        let status: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Status)
            .expect("not initialised");
        assert_eq!(status, STATUS_FUNDING, "can only cancel during Funding phase");

        env.storage()
            .instance()
            .set(&DataKey::Status, &STATUS_CANCELLED);

        let funded_amount: i128 = env
            .storage()
            .instance()
            .get(&DataKey::FundedAmount)
            .unwrap_or(0);

        env.events().publish(
            (soroban_sdk::symbol_short!("cancelled"),),
            FundingCancelled {
                admin,
                funded_amount,
            },
        );
    }

    // ── Investor: refund ──────────────────────────────────────────────────────

    /// Return an investor's contribution after a cancellation.
    ///
    /// # Access control
    /// Requires `investor` authorisation.
    ///
    /// # Preconditions
    /// - Status must be `4` (Cancelled).
    /// - `investor` must have a non-zero recorded contribution.
    ///
    /// # Effects
    /// - Zeroes `InvestorContribution(investor)` (prevents double-refund).
    /// - Transfers exactly the recorded contribution back to `investor` via
    ///   [`external_calls::transfer_funding_token_with_balance_checks`].
    /// - Emits [`Refunded`].
    pub fn refund(env: Env, investor: Address) {
        investor.require_auth();

        let status: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Status)
            .expect("not initialised");
        assert_eq!(status, STATUS_CANCELLED, "refunds only available after cancellation");

        let contribution_key = DataKey::InvestorContribution(investor.clone());
        let amount: i128 = env
            .storage()
            .persistent()
            .get(&contribution_key)
            .unwrap_or(0);
        assert!(amount > 0, "no contribution to refund");

        // Zero out before transfer — prevents re-entrancy / double-spend.
        env.storage()
            .persistent()
            .set(&contribution_key, &0_i128);

        let funding_token: Address = env
            .storage()
            .instance()
            .get(&DataKey::FundingToken)
            .expect("not initialised");

        external_calls::transfer_funding_token_with_balance_checks(
            &env,
            &funding_token,
            &env.current_contract_address(),
            &investor,
            amount,
        );

        env.events().publish(
            (soroban_sdk::symbol_short!("refunded"),),
            Refunded {
                investor,
                amount,
            },
        );
    }

    // ── Investor: contribute ──────────────────────────────────────────────────

    /// Record an investor contribution during the Funding phase.
    ///
    /// Transfers `amount` from `investor` to this contract and records it.
    ///
    /// # Preconditions
    /// - Status must be `0` (Funding).
    /// - `amount` must be positive.
    pub fn contribute(env: Env, investor: Address, amount: i128) {
        investor.require_auth();
        assert!(amount > 0, "amount must be positive");

        let status: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Status)
            .expect("not initialised");
        assert_eq!(status, STATUS_FUNDING, "contributions only accepted during Funding phase");

        let funding_token: Address = env
            .storage()
            .instance()
            .get(&DataKey::FundingToken)
            .expect("not initialised");

        let token_client = token::Client::new(&env, &funding_token);
        token_client.transfer(&investor, &env.current_contract_address(), &amount);

        let contribution_key = DataKey::InvestorContribution(investor.clone());
        let prev: i128 = env
            .storage()
            .persistent()
            .get(&contribution_key)
            .unwrap_or(0);
        let new_contribution = prev.checked_add(amount).expect("contribution overflow");
        env.storage()
            .persistent()
            .set(&contribution_key, &new_contribution);

        let prev_funded: i128 = env
            .storage()
            .instance()
            .get(&DataKey::FundedAmount)
            .unwrap_or(0);
        let new_funded = prev_funded.checked_add(amount).expect("funded_amount overflow");
        env.storage()
            .instance()
            .set(&DataKey::FundedAmount, &new_funded);

        env.storage()
            .persistent()
            .extend_ttl(&contribution_key, 17_280, 17_280 * 30);
    }

    // ── View helpers ──────────────────────────────────────────────────────────

    /// Returns the current lifecycle status code.
    pub fn status(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::Status)
            .unwrap_or(STATUS_FUNDING)
    }

    /// Returns the recorded contribution for `investor`, or `0`.
    pub fn contribution(env: Env, investor: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::InvestorContribution(investor))
            .unwrap_or(0)
    }

    /// Returns the total funded amount.
    pub fn funded_amount(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::FundedAmount)
            .unwrap_or(0)
    }
}
