#!/usr/bin/env bash
set -euo pipefail

stellar contract build --optimize=false --locked
cmp fixtures/wasm/fleet_v1.wasm target/wasm32v1-none/release/fleet_v1.wasm
cmp fixtures/wasm/fleet_v2_compatible.wasm target/wasm32v1-none/release/fleet_v2_compatible.wasm
cmp fixtures/wasm/fleet_v2_breaking.wasm target/wasm32v1-none/release/fleet_v2_breaking.wasm
cmp fixtures/wasm/fleet_v2_migration.wasm target/wasm32v1-none/release/fleet_v2_migration.wasm
stellar contract build --locked
