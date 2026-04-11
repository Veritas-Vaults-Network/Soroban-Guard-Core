# Soroban Guard Core

Static analyzer CLI for [Soroban](https://soroban.stellar.org/) smart contracts (Rust). This repository is the **core engine** in a three-repo setup:

| Repo | URL |
|------|-----|
| **Core** (this) | [github.com/Veritas-Vaults-Network/soroban-guard-core](https://github.com/Veritas-Vaults-Network/soroban-guard-core) |
| **Web dashboard** | [github.com/Veritas-Vaults-Network/Soroban-Guard-web](https://github.com/Veritas-Vaults-Network/Soroban-Guard-web) |
| **Contracts** | [github.com/Veritas-Vaults-Network/soroban-guard-contracts](https://github.com/Veritas-Vaults-Network/soroban-guard-contracts) |

## Requirements

- Rust 1.74+ (2021 edition)

## Build

```bash
cargo build --release
```

The binary is `target/release/soroban-guard` (package `soroban-guard-cli`).

## Usage

```bash
cargo run -p soroban-guard-cli -- scan ./path/to/contract-crate
```

Pretty-printed, colored findings on stdout; JSON with:

```bash
cargo run -p soroban-guard-cli -- scan ./path/to/contract-crate --json
```

- Exit **0**: no **High** severity findings (Medium/Low do not fail the process).
- Exit **1**: at least one **High** finding.
- Exit **2**: scan error (I/O or parse failure).

## Workspace layout

| Crate | Role |
|-------|------|
| `crates/cli` | `clap` entrypoint, reporting |
| `crates/analyzer` | Walk `.rs` files, parse with `syn`, run checks |
| `crates/checks` | `Check` trait + individual detectors |

See [docs/checks.md](docs/checks.md) for implemented rules and [CONTRIBUTING.md](CONTRIBUTING.md) to add a check.

## Test contracts

Sample crates under `test-contracts/` are listed in the root workspace `exclude` list so they stay standalone Soroban packages. The CLI scans their `.rs` sources directly (no need to `cargo build` them first):

| Directory | Intent |
|-----------|--------|
| `vulnerable` / `safe` | Phase 1 — `missing-require-auth` |
| `arithmetic-vulnerable` / `arithmetic-safe` | Phase 2 — `unchecked-arithmetic` |
| `admin-vulnerable` / `admin-safe` | Phase 2 — `unprotected-admin` |
| `storage-vulnerable` / `storage-safe` | Phase 2 — `unsafe-storage-patterns` |

## License

MIT OR Apache-2.0 (see workspace `Cargo.toml`).
