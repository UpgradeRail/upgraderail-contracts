# UpgradeRail Contracts

UpgradeRail is an open-source release safety system for Soroban contracts. This repository contains the on-chain governance and execution layer. Its only production contract is `UpgradeController`. The contract owns CAP-85 executable references and changes them only after a proposal, approver threshold, and timelock. It does not analyze WASM or run simulations.

## Why CAP-85 matters

A Soroban contract can use an executable reference owned by another contract. The owner stores a tag that points to an uploaded WASM hash. Changing that reference changes the executable used by every contract attached to it at the next invocation. `UpgradeController` owns these references for fleets. It stores a fleet ID and tag, while fleet membership is tracked off-chain.

```text
approvers -> proposal and manifest hash -> threshold -> timelock
                                                    -> permissionless execution
                                                    -> CAP-85 reference update
                                                    -> all attached instances
```

`upgraderail-engine` produces analysis and release manifest commitments. `upgraderail-console` provides the API, indexer, team management, and user interface. The controller stores a 32-byte manifest hash so clients can connect an executed change to the report the approvers reviewed. It does not interpret or endorse the report.

## Governance lifecycle

The constructor installs a `GovernancePolicy`, starts governance epoch and controller version at 1, and creates no administrator. A policy has 1 to 20 unique approvers, a threshold within that count, a positive timelock, and a proposal lifetime longer than the timelock. The lifetime plus a 17,280-ledger safety buffer must fit the supported TTL maximum. Fleet tags are limited to 1 to 64 bytes by UpgradeRail policy.

A current approver creates a proposal and signs that transaction. Creation does not count as approval. Each approver signs a separate `approve` call. The timelock starts when the approval count first reaches the threshold. Revoking an approval below the threshold clears the timelock; reaching it again starts a new one. The proposer may cancel while the proposal is active and below the threshold.

Anyone may call `execute_proposal` once the timelock has elapsed and before the expiry ledger. Execution checks the current governance epoch, approval count, and action-specific state again. Fleet upgrades require the current WASM to equal the proposal's expected hash. A policy update increments the epoch, making older unfinished proposals stale. A controller WASM upgrade uses the same governance path and increments the controller version by one.

Proposal state is derived in this order: executed, cancelled, stale, expired, awaiting approvals, timelocked, ready. Expiry begins at `expires_ledger`. Failed execution leaves the proposal and executable reference unchanged.

## Public interface

| Area | Functions |
| --- | --- |
| Initialization | `__constructor` |
| Governance | `create_proposal`, `approve`, `revoke_approval`, `cancel_proposal`, `execute_proposal` |
| Reads | `get_policy`, `get_governance_epoch`, `get_controller_version`, `get_proposal`, `get_proposal_state`, `has_approved`, `get_fleet`, `get_current_wasm` |
| TTL maintenance | `maintain_controller`, `maintain_fleet`, `maintain_proposal` |

The four proposal kinds are `CreateFleet`, `UpgradeFleet`, `UpdatePolicy`, and `UpgradeController`. The controller emits typed events for proposal creation, approvals, threshold changes, cancellation, execution, fleet changes, policy updates, and controller upgrades. Maintenance and execution require no authorization; neither grants governance authority.

## Build and test

Use Rust 1.98.1, `wasm32v1-none`, Soroban SDK 28.0.0, and Stellar CLI 28.1.0. The workspace pins the SDK and commits `Cargo.lock`. Install the Rust target, then run:

```sh
make ci
```

This runs formatting, workspace compilation, Clippy with warnings denied, all tests, and WASM builds. `scripts/build.sh` first builds unoptimized WASM and compares every fixture to its committed test binary, then runs the standard optimized `stellar contract build`. The deployable controller is `target/wasm32v1-none/release/upgrade_controller.wasm` after the optimized build. Tests upload real fixture WASM to a local Soroban environment and verify that two instances sharing a CAP-85 reference both change behavior after an upgrade. A separate fixture exercises an explicit state migration.

Run a single task with `make fmt`, `make check`, `make test`, `make build`, or `make clippy`. See [CONTRIBUTING.md](CONTRIBUTING.md) when editing fixtures or governance code.

## Testnet deployment

The deployment scripts use a Stellar CLI identity. Set `STELLAR_SOURCE` to a funded identity name and `UPGRADERAIL_POLICY_JSON` to a real `GovernancePolicy` JSON value. `.env.example` lists these variables and contains no secrets. From a clean committed tree, run:

```sh
./scripts/deploy-testnet.sh
```

The script builds the controller, deploys it with the policy constructor argument, checks its WASM hash and initial version, then writes a local `deployments/testnet.json` record after success. It leaves unavailable transaction and deployment ledger fields null rather than inventing evidence. `scripts/verify-deployment.sh` can check an initial deployment again when given its contract ID and expected hash.

**Testnet reported Protocol 29 on October 1, 2026.** This repository and Stellar CLI are pinned to Protocol 28. The deployment script currently stops before submitting any transaction. No Testnet or Mainnet deployment is claimed. A Protocol 29 migration requires separate compatibility work and validation.

## Security and limits

There is no emergency administrator. Approver authorization and on-chain state checks govern executable changes. Anyone may pay for TTL maintenance, but the uploaded WASM code has its own TTL and operators must maintain it as well. The manifest hash is a commitment, not an on-chain safety verdict. The code has not had an independent audit. See [SECURITY.md](SECURITY.md) for the trust model and reporting process.

## License

Apache License 2.0. See [LICENSE](LICENSE).
