# Contributing to Soroban Guard Core

Thank you for helping improve the static analyzer. This guide covers **local setup**, a **short `syn` tutorial with examples**, **how to add a check** (using `auth.rs` as a template), **how to write test contracts**, and links to **sister repositories** in the Veritas Vaults ecosystem.

## Local development setup

1. **Install Rust** (1.74 or newer recommended) using [rustup](https://rustup.rs/):

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source "$HOME/.cargo/env"
   rustc --version
   ```
