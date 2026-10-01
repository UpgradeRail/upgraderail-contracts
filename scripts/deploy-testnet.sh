#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

stellar_version=$(stellar version)
if [[ "$stellar_version" != "stellar 28.1.0 "* ]]; then
    echo "This deployment requires Stellar CLI 28.1.0." >&2
    exit 1
fi

ledger_json=$(stellar ledger latest --network testnet --output json)
protocol=$(python3 -c 'import json,sys; print(json.loads(sys.argv[1])["protocolVersion"])' "$ledger_json")
# SDK 28 WASM and CLI 28.1.0 completed a live Protocol 29 Testnet upgrade flow on 2026-10-01.
if [[ "$protocol" != "28" && "$protocol" != "29" ]]; then
    echo "Testnet is at unverified Protocol $protocol; deployment requires a new compatibility check." >&2
    exit 1
fi

: "${STELLAR_SOURCE:?Set STELLAR_SOURCE to a funded Stellar CLI identity}"
: "${UPGRADERAIL_POLICY_JSON:?Set UPGRADERAIL_POLICY_JSON to a real GovernancePolicy JSON value}"
python3 -c 'import json,sys; json.loads(sys.argv[1])' "$UPGRADERAIL_POLICY_JSON"
record_path=${UPGRADERAIL_RECORD_PATH:-deployments/testnet.local.json}
if [[ -e "$record_path" ]]; then
    echo "Deployment record $record_path already exists; choose a new path." >&2
    exit 1
fi

if [[ -n "$(git status --porcelain)" ]]; then
    echo "Commit local changes before deployment so the source commit is truthful." >&2
    exit 1
fi

./scripts/build.sh
wasm=target/wasm32v1-none/release/upgrade_controller.wasm
wasm_hash=$(stellar contract info hash --wasm "$wasm")
source_commit=$(git rev-parse HEAD)
contract_id=$(stellar contract deploy \
    --network testnet \
    --source "$STELLAR_SOURCE" \
    --wasm "$wasm" \
    -- --policy "$UPGRADERAIL_POLICY_JSON")
if [[ ! "$contract_id" =~ ^C[A-Z2-7]{55}$ ]]; then
    echo "Deployment did not return a valid contract ID." >&2
    exit 1
fi

./scripts/verify-deployment.sh "$contract_id" "$wasm_hash"
controller_version=$(stellar contract invoke \
    --network testnet \
    --source "$STELLAR_SOURCE" \
    --contract-id "$contract_id" \
    --send=no \
    -- get_controller_version)
observed_ledger_json=$(stellar ledger latest --network testnet --output json)

python3 - "$record_path" "$contract_id" "$wasm_hash" "$source_commit" "$controller_version" "$observed_ledger_json" <<'PY'
import datetime
import json
import pathlib
import sys

record_path, contract_id, wasm_hash, source_commit, version, ledger_json = sys.argv[1:]
record = {
    "network": "testnet",
    "contract_id": contract_id,
    "wasm_hash": wasm_hash,
    "transaction_hash": None,
    "deployment_ledger": None,
    "observed_ledger_after_verification": json.loads(ledger_json)["sequence"],
    "deployment_completed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "source_commit": source_commit,
    "controller_version": int(version),
}
pathlib.Path(record_path).write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps(record, indent=2))
PY
