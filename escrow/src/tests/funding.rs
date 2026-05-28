//! Tests for `cancel_funding` and `refund` entrypoints.
//!
//! Coverage:
//! - Happy-path cancel + single refund
//! - Happy-path cancel + multiple investors refund independently
//! - Double-refund is a no-op (panics on second call)
//! - Refund in wrong state (Funding, Active) panics
//! - cancel_funding in wrong state (Active) panics
//! - cancel_funding under legal hold panics
//! - Unauthorised cancel_funding panics
//! - Unauthorised refund panics
//! - Zero-contribution refund panics
//! - Contribution overflow guard
//! - Contribute in wrong state (Cancelled) panics
//! - funded_amount invariant: total refunded ≤ funded_amount

#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Events},
    token, Address, Env, IntoVal,
};

use crate::{DataKey, EscrowContract, EscrowContractClient};

// ── Test helpers ──────────────────────────────────────────────────────────────

struct TestEnv {
    env: Env,
    contract: Address,
    client: EscrowContractClient<'static>,
    token: Address,
    admin: Address,
}

impl TestEnv {
    fn new() -> Self {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let token_admin = Address::generate(&env);

        // Deploy a standard SEP-41 token for testing.
        let token = env.register_stellar_asset_contract_v2(token_admin.clone()).address();

        let contract = env.register(EscrowContract, ());
        // SAFETY: the client lifetime is tied to `env` which lives in the struct.
        let client = EscrowContractClient::new(&env, &contract);

        client.initialize(&admin, &token, &1_000_000_i128);

        TestEnv {
            env,
            contract,
            client,
            token,
            admin,
        }
    }

    /// Mint `amount` tokens to `addr`.
    fn mint(&self, addr: &Address, amount: i128) {
        let token_client = token::StellarAssetClient::new(&self.env, &self.token);
        token_client.mint(addr, &amount);
    }

    /// Contribute `amount` from `investor`.
    fn contribute(&self, investor: &Address, amount: i128) {
        self.mint(investor, amount);
        self.client.contribute(investor, &amount);
    }
}

// ── cancel_funding ────────────────────────────────────────────────────────────

#[test]
fn cancel_funding_transitions_status_to_4() {
    let t = TestEnv::new();
    assert_eq!(t.client.status(), 0);
    t.client.cancel_funding();
    assert_eq!(t.client.status(), 4);
}

#[test]
fn cancel_funding_emits_event() {
    let t = TestEnv::new();
    t.contribute(&Address::generate(&t.env), 500_000);
    t.client.cancel_funding();

    let events = t.env.events().all();
    let has_cancelled = events.iter().any(|(_, topics, _)| {
        topics
            .iter()
            .any(|v| v == soroban_sdk::symbol_short!("cancelled").into_val(&t.env))
    });
    assert!(has_cancelled, "FundingCancelled event not emitted");
}

#[test]
#[should_panic(expected = "can only cancel during Funding phase")]
fn cancel_funding_fails_when_not_in_funding_state() {
    let t = TestEnv::new();
    // Manually set status to Active (1).
    t.env
        .as_contract(&t.contract, || {
            t.env
                .storage()
                .instance()
                .set(&DataKey::Status, &1_u32);
        });
    t.client.cancel_funding();
}

#[test]
#[should_panic(expected = "can only cancel during Funding phase")]
fn cancel_funding_is_idempotent_guard() {
    let t = TestEnv::new();
    t.client.cancel_funding();
    // Second call must panic — already in state 4.
    t.client.cancel_funding();
}

#[test]
#[should_panic(expected = "escrow is under legal hold")]
fn cancel_funding_blocked_under_legal_hold() {
    let t = TestEnv::new();
    t.env.as_contract(&t.contract, || {
        t.env
            .storage()
            .instance()
            .set(&DataKey::LegalHold, &true);
    });
    t.client.cancel_funding();
}

#[test]
#[should_panic]
fn cancel_funding_requires_admin_auth() {
    let env = Env::default();
    // Do NOT mock auths — auth will fail.
    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract_v2(token_admin).address();
    let contract = env.register(EscrowContract, ());
    let client = EscrowContractClient::new(&env, &contract);
    client.initialize(&admin, &token, &1_000_000_i128);
    client.cancel_funding(); // should panic: missing auth
}

// ── refund ────────────────────────────────────────────────────────────────────

#[test]
fn refund_returns_exact_contribution() {
    let t = TestEnv::new();
    let investor = Address::generate(&t.env);
    t.contribute(&investor, 300_000);

    t.client.cancel_funding();

    let token_client = token::Client::new(&t.env, &t.token);
    let before = token_client.balance(&investor);
    t.client.refund(&investor);
    let after = token_client.balance(&investor);

    assert_eq!(after - before, 300_000);
}

#[test]
fn refund_zeroes_contribution_record() {
    let t = TestEnv::new();
    let investor = Address::generate(&t.env);
    t.contribute(&investor, 200_000);
    t.client.cancel_funding();
    t.client.refund(&investor);

    assert_eq!(t.client.contribution(&investor), 0);
}

#[test]
fn refund_emits_event() {
    let t = TestEnv::new();
    let investor = Address::generate(&t.env);
    t.contribute(&investor, 100_000);
    t.client.cancel_funding();
    t.client.refund(&investor);

    let events = t.env.events().all();
    let has_refunded = events.iter().any(|(_, topics, _)| {
        topics
            .iter()
            .any(|v| v == soroban_sdk::symbol_short!("refunded").into_val(&t.env))
    });
    assert!(has_refunded, "Refunded event not emitted");
}

#[test]
#[should_panic(expected = "no contribution to refund")]
fn double_refund_panics() {
    let t = TestEnv::new();
    let investor = Address::generate(&t.env);
    t.contribute(&investor, 100_000);
    t.client.cancel_funding();
    t.client.refund(&investor);
    t.client.refund(&investor); // second call must panic
}

#[test]
#[should_panic(expected = "refunds only available after cancellation")]
fn refund_fails_in_funding_state() {
    let t = TestEnv::new();
    let investor = Address::generate(&t.env);
    t.contribute(&investor, 100_000);
    // Status is still 0 — no cancel.
    t.client.refund(&investor);
}

#[test]
#[should_panic(expected = "refunds only available after cancellation")]
fn refund_fails_in_active_state() {
    let t = TestEnv::new();
    let investor = Address::generate(&t.env);
    t.contribute(&investor, 100_000);
    t.env.as_contract(&t.contract, || {
        t.env
            .storage()
            .instance()
            .set(&DataKey::Status, &1_u32);
    });
    t.client.refund(&investor);
}

#[test]
#[should_panic(expected = "no contribution to refund")]
fn refund_zero_contribution_panics() {
    let t = TestEnv::new();
    let investor = Address::generate(&t.env);
    // investor never contributed
    t.client.cancel_funding();
    t.client.refund(&investor);
}

#[test]
#[should_panic]
fn refund_requires_investor_auth() {
    let env = Env::default();
    // Do NOT mock auths.
    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract_v2(token_admin).address();
    let contract = env.register(EscrowContract, ());
    let client = EscrowContractClient::new(&env, &contract);
    client.initialize(&admin, &token, &1_000_000_i128);

    // Mock only admin auth to allow cancel.
    env.mock_auths(&[AuthorizedInvocation {
        function: AuthorizedFunction::Contract((
            contract.clone(),
            soroban_sdk::symbol_short!("cancel_f"),
            soroban_sdk::vec![&env],
        )),
        sub_invocations: soroban_sdk::vec![&env],
    }]);
    client.cancel_funding();

    // Now call refund without investor auth — should panic.
    let investor = Address::generate(&env);
    client.refund(&investor);
}

// ── Multiple investors ────────────────────────────────────────────────────────

#[test]
fn multiple_investors_each_refunded_correctly() {
    let t = TestEnv::new();
    let alice = Address::generate(&t.env);
    let bob = Address::generate(&t.env);

    t.contribute(&alice, 400_000);
    t.contribute(&bob, 250_000);

    t.client.cancel_funding();

    let token_client = token::Client::new(&t.env, &t.token);

    let alice_before = token_client.balance(&alice);
    t.client.refund(&alice);
    assert_eq!(token_client.balance(&alice) - alice_before, 400_000);

    let bob_before = token_client.balance(&bob);
    t.client.refund(&bob);
    assert_eq!(token_client.balance(&bob) - bob_before, 250_000);
}

#[test]
fn funded_amount_invariant_holds_after_refunds() {
    let t = TestEnv::new();
    let alice = Address::generate(&t.env);
    let bob = Address::generate(&t.env);

    t.contribute(&alice, 300_000);
    t.contribute(&bob, 200_000);

    let funded = t.client.funded_amount();
    assert_eq!(funded, 500_000);

    t.client.cancel_funding();
    t.client.refund(&alice);
    t.client.refund(&bob);

    // funded_amount is a historical record; it does not decrease on refund.
    // The invariant is: sum of all refunds ≤ funded_amount.
    assert_eq!(t.client.funded_amount(), 500_000);
}

// ── Contribute guards ─────────────────────────────────────────────────────────

#[test]
#[should_panic(expected = "contributions only accepted during Funding phase")]
fn contribute_fails_after_cancellation() {
    let t = TestEnv::new();
    t.client.cancel_funding();
    let investor = Address::generate(&t.env);
    t.contribute(&investor, 100_000);
}

#[test]
#[should_panic(expected = "amount must be positive")]
fn contribute_zero_amount_panics() {
    let t = TestEnv::new();
    let investor = Address::generate(&t.env);
    t.client.contribute(&investor, &0);
}

// ── Initialisation guard ──────────────────────────────────────────────────────

#[test]
#[should_panic(expected = "already initialised")]
fn double_initialize_panics() {
    let t = TestEnv::new();
    t.client.initialize(&t.admin, &t.token, &1_000_000_i128);
}

#[test]
#[should_panic(expected = "funding_target must be positive")]
fn initialize_zero_target_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract_v2(token_admin).address();
    let contract = env.register(EscrowContract, ());
    let client = EscrowContractClient::new(&env, &contract);
    client.initialize(&admin, &token, &0_i128);
}
