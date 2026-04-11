# Soroban Guard Core

Static analyzer CLI for [Soroban](https://soroban.stellar.org/) smart contracts (Rust). This repository is the **core engine**; companion repos host the web dashboard and curated contracts under [Veritas-Vaults-Network](https://github.com/Veritas-Vaults-Network).

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

## License

MIT OR Apache-2.0 (see workspace `Cargo.toml`).
