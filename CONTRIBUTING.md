# Contributing to Soroban Guard Core

Thank you for helping improve the static analyzer. This guide covers **local setup**, a **short `syn` tutorial with examples**, **how to add a check** (using `auth.rs` as a template), **how to write test contracts**, and links to **sister repositories** in the Veritas Vaults ecosystem.

## Local development setup

1. **Install Rust** (1.74 or newer recommended) using [rustup](https://rustup.rs/):

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source "$HOME/.cargo/env"
   rustc --version
   ```

2. **Clone this repository** and `cd` into the workspace root:

   ```bash
   git clone https://github.com/Veritas-Vaults-Network/soroban-guard-core.git
   cd soroban-guard-core
   ```

3. **Build and run the full test suite:**

   ```bash
   cargo build --workspace
   cargo test --workspace
   ```

4. **Run the CLI** against bundled fixtures:

   ```bash
   cargo run -p soroban-guard-cli -- scan test-contracts/vulnerable
   cargo run -p soroban-guard-cli -- scan test-contracts/safe --json
   ```

5. **Standalone Soroban crates** — Paths under `test-contracts/` are listed in `[workspace.exclude]`. To type-check a fixture on its own:

   ```bash
   cd test-contracts/arithmetic-safe && cargo check
   ```

6. **Install the `soroban-guard` binary** (optional):

   ```bash
   cargo install --path crates/cli
