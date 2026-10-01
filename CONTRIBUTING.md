# Contributing to UpgradeRail Contracts

This repository contains one production contract, `UpgradeController`, four executable fixtures, and one external-reference factory used for live verification. Keep changes focused on governed CAP-85 upgrades. Static analysis, simulation, the API, and the console live in other UpgradeRail repositories.

## Toolchain

Use Rust 1.98.1 with `wasm32v1-none`, Soroban SDK 28.0.0, and Stellar CLI 28.1.0. `rust-toolchain.toml` selects the Rust version. Do not change these versions simply to resolve a local build problem. Verify compatibility against the SDK source and the target network first.

The workspace has a committed lockfile. Install the toolchain and CLI, then run:

```sh
make ci
```

This checks formatting, compilation, Clippy, tests, WASM builds, and the committed fixture binaries. Run it before pushing. `make fmt`, `make check`, `make test`, `make build`, and `make clippy` run the individual tasks.

## Contract changes

- Preserve proposal authorization, epoch checks, expected hash checks, and the timelock on every path that changes an executable reference or the controller WASM.
- Keep arithmetic checked. Keep loops bounded by the policy's tested 20-approver limit.
- Use typed storage keys and the SDK's `executable_refs()` API. A regular persistent storage entry is not a CAP-85 reference.
- Add tests for successful and rejected transitions. Use uploaded fixture WASM when the behavior depends on actual executable references.
- Preserve stable numeric contract error codes and event fields consumed by an indexer.
- Keep TTL maintenance aligned for proposal and approval records, fleet metadata, and executable references.

## Fixture binaries

Tests load the committed binaries in `fixtures/wasm`. These files are intentional test assets. When fixture source changes, run `stellar contract build --optimize=false`, copy the matching WASM from `target/wasm32v1-none/release` into `fixtures/wasm`, and rerun `make ci`. The build script compares each fixture binary with its source build before the optimized build. The controller v1 binary is a fixed upgrade test fixture, not the deployment artifact.

## Testnet deployment

The deployment script requires `STELLAR_SOURCE` to name a funded Stellar CLI identity and `UPGRADERAIL_POLICY_JSON` to contain the real constructor policy. Do not put secret keys or seed phrases in the repository. It checks the live Testnet protocol and accepts Protocol 28 or the Protocol 29 path verified on October 1, 2026, using CLI 28.1.0 and SDK 28 WASM. A later protocol requires another compatibility check. See the [Testnet deployment record](deployments/testnet.json) for the real Protocol 29 verification. Mainnet remains a Protocol 28 target.

## Git and review

Use a focused conventional commit for each logical change. Review the staged diff before committing, then push the commit. Use your configured Git identity and do not add an agent co-author. Include the behavior changed, tests run, and any remaining limits in the review description. See [SECURITY.md](SECURITY.md) before changing governance or deployment behavior.
