# Security policy

## Supported version

The v1 development line uses Soroban SDK 28.0.0 and targets Mainnet Protocol 28. There is no Mainnet deployment in this repository. The SDK 28 WASM was deployed and exercised on Protocol 29 Testnet on October 1, 2026; see [deployment evidence](deployments/testnet.json). The Testnet policy is for verification only.

## Reporting a vulnerability

Security contact: **project maintainers must add a private reporting address before production deployment**. Until then, contact an UpgradeRail maintainer privately through the project's established communication channel. Do not publish an exploit or sensitive details in a public issue. Include the affected commit, reproduction steps, and expected impact. Maintainers should acknowledge the report, investigate it, and coordinate a fix and disclosure date with the reporter.

## Audit status

No independent security audit has been completed. Passing tests and WASM builds are validation evidence, not an audit.

## Trust model and assumptions

- `UpgradeController` owns CAP-85 executable references. A reference change requires a proposal, the current policy's approval threshold, and a ledger-based timelock. There is no owner key, emergency administrator, or direct upgrade entry point.
- A policy update increments the governance epoch. Unfinished proposals from earlier epochs cannot be approved or executed.
- Fleet upgrades compare the current reference hash with the proposal's expected hash immediately before changing it. A competing upgrade makes an older proposal fail.
- Controller WASM upgrades use the same proposal path. The new code receives all existing controller state. Approvers must review compatibility and migration effects before approving it.
- The manifest hash commits to a report identified by the off-chain engine. The controller does not parse that report or certify its safety. Approvers and clients must independently verify that the reviewed report matches the on-chain hash.
- Approval records and proposal records must remain live together until the proposal expires. The contract extends their TTL, and permissionless maintenance methods allow any fee payer to extend them. Archived entries require restoration before use.
- Fleet metadata and CAP-85 reference entries have coordinated TTL maintenance. Uploaded WASM code has its own TTL and must also be maintained by operators. Keeping the reference live alone does not guarantee that the code remains available.
- Soroban authorization is checked with `require_auth` for proposal creation, approval, revocation, and cancellation. Execution and TTL maintenance are permissionless by design.
- A deployment operator must confirm the live network protocol and configuration. The SDK 28 WASM was validated on Protocol 29 Testnet with Stellar CLI 28.1.0, but a later protocol requires a fresh compatibility check.

## Timelock and expiry

The timelock starts when approvals first reach the threshold. If revocation drops the count below the threshold, the recorded execution ledger is cleared. A later threshold starts a new timelock. A proposal expires at its `expires_ledger`; it cannot execute on that ledger. Ledger duration is a network setting, so wall-clock estimates are approximate.
