use soroban_sdk::{BytesN, Env};

use crate::errors::ContractError;
use crate::events::{FleetCreated, FleetUpgraded};
use crate::storage;
use crate::types::{CreateFleetProposal, DataKey, Fleet, UpgradeFleetProposal};

pub(crate) fn current_wasm(env: &Env, id: &BytesN<32>) -> Result<BytesN<32>, ContractError> {
    let fleet = storage::fleet(env, id).ok_or(ContractError::FleetNotFound)?;
    env.executable_refs()
        .get(&fleet.tag)
        .ok_or(ContractError::ExecutableRefMissing)
}

pub(crate) fn create(
    env: &Env,
    proposal_id: u64,
    payload: &CreateFleetProposal,
) -> Result<(), ContractError> {
    if storage::fleet(env, &payload.fleet_id).is_some() {
        return Err(ContractError::FleetExists);
    }
    if storage::tag_used(env, &payload.tag) || env.executable_refs().has(&payload.tag) {
        return Err(ContractError::TagAlreadyUsed);
    }
    env.executable_refs()
        .set(&payload.tag, &payload.initial_wasm_hash);
    let fleet = Fleet {
        id: payload.fleet_id.clone(),
        tag: payload.tag.clone(),
        created_ledger: env.ledger().sequence(),
    };
    storage::set_fleet(env, &fleet);
    let max_ttl = env.storage().max_ttl();
    env.storage().persistent().extend_ttl_with_limits(
        &DataKey::Fleet(fleet.id.clone()),
        max_ttl,
        1,
        max_ttl,
    );
    env.storage().persistent().extend_ttl_with_limits(
        &DataKey::FleetByTag(fleet.tag.clone()),
        max_ttl,
        1,
        max_ttl,
    );
    env.executable_refs()
        .extend_ttl_with_limits(&fleet.tag, max_ttl, 1, max_ttl);
    FleetCreated {
        fleet_id: fleet.id,
        proposal_id,
        tag: fleet.tag,
        wasm_hash: payload.initial_wasm_hash.clone(),
        manifest_hash: payload.manifest_hash.clone(),
        ledger: env.ledger().sequence(),
    }
    .publish(env);
    Ok(())
}

pub(crate) fn upgrade(
    env: &Env,
    proposal_id: u64,
    payload: &UpgradeFleetProposal,
) -> Result<(), ContractError> {
    let fleet = storage::fleet(env, &payload.fleet_id).ok_or(ContractError::FleetNotFound)?;
    let current = env
        .executable_refs()
        .get(&fleet.tag)
        .ok_or(ContractError::ExecutableRefMissing)?;
    if current != payload.expected_wasm_hash {
        return Err(ContractError::CurrentWasmMismatch);
    }
    if current == payload.new_wasm_hash {
        return Err(ContractError::CandidateMatchesCurrent);
    }
    env.executable_refs()
        .set(&fleet.tag, &payload.new_wasm_hash);
    FleetUpgraded {
        fleet_id: fleet.id,
        proposal_id,
        old_wasm_hash: current,
        new_wasm_hash: payload.new_wasm_hash.clone(),
        manifest_hash: payload.manifest_hash.clone(),
    }
    .publish(env);
    Ok(())
}
