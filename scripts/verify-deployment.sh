#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

contract_id=${1:?Pass the deployed Testnet contract ID}
expected_hash=${2:?Pass the expected WASM hash}
: "${STELLAR_SOURCE:?Set STELLAR_SOURCE to a Stellar CLI identity}"

actual_hash=$(stellar contract info hash --network testnet --contract-id "$contract_id")
if [[ "$actual_hash" != "$expected_hash" ]]; then
    echo "Deployed WASM hash does not match the expected hash." >&2
    exit 1
fi

version=$(stellar contract invoke \
    --network testnet \
    --source "$STELLAR_SOURCE" \
    --contract-id "$contract_id" \
    --send=no \
    -- get_controller_version)
epoch=$(stellar contract invoke \
    --network testnet \
    --source "$STELLAR_SOURCE" \
    --contract-id "$contract_id" \
    --send=no \
    -- get_governance_epoch)
if [[ "$version" != "1" || "$epoch" != "1" ]]; then
    echo "Deployed controller did not return its expected initial version and epoch." >&2
    exit 1
fi

echo "Verified Testnet controller $contract_id at version $version and governance epoch $epoch."
