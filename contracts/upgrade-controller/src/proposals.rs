use soroban_sdk::{Address, BytesN, Env, String};

use crate::errors::ContractError;
use crate::events::ProposalCreated;
use crate::policy::{self, PROPOSAL_TTL_SAFETY_BUFFER};
use crate::storage;
use crate::types::{DataKey, Proposal, ProposalKind, StoredProposalStatus};

pub const MAX_FLEET_TAG_BYTES: u32 = 64;

fn nonzero(env: &Env, value: &BytesN<32>) -> bool {
    *value != BytesN::from_array(env, &[0; 32])
}

fn valid_tag(tag: &String) -> bool {
    let length = tag.len();
    length > 0 && length <= MAX_FLEET_TAG_BYTES
}

fn validate_kind(env: &Env, kind: &ProposalKind) -> Result<(), ContractError> {
    match kind {
        ProposalKind::CreateFleet(payload) => {
            if !valid_tag(&payload.tag) {
                return Err(ContractError::InvalidFleetTag);
            }
            if !nonzero(env, &payload.initial_wasm_hash) {
                return Err(ContractError::InvalidWasmHash);
            }
            if !nonzero(env, &payload.manifest_hash) {
                return Err(ContractError::InvalidManifestHash);
            }
            if storage::fleet(env, &payload.fleet_id).is_some() {
                return Err(ContractError::FleetExists);
            }
            if storage::tag_used(env, &payload.tag) || env.executable_refs().has(&payload.tag) {
                return Err(ContractError::TagAlreadyUsed);
            }
        }
        ProposalKind::UpgradeFleet(payload) => {
            if !nonzero(env, &payload.new_wasm_hash) {
                return Err(ContractError::InvalidWasmHash);
            }
            if !nonzero(env, &payload.manifest_hash) {
                return Err(ContractError::InvalidManifestHash);
            }
            let fleet = storage::fleet(env, &payload.fleet_id)
                .ok_or(ContractError::FleetNotFound)?;
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
        }
        ProposalKind::UpdatePolicy(payload) => policy::validate(env, &payload.policy)?,
        ProposalKind::UpgradeController(payload) => {
            if !nonzero(env, &payload.new_wasm_hash) {
                return Err(ContractError::InvalidWasmHash);
            }
            if !nonzero(env, &payload.manifest_hash) {
                return Err(ContractError::InvalidManifestHash);
            }
            let current = storage::version(env).ok_or(ContractError::InvalidPolicy)?;
            if current != payload.expected_controller_version {
                return Err(ContractError::ControllerVersionMismatch);
            }
            let next = current
                .checked_add(1)
                .ok_or(ContractError::ArithmeticOverflow)?;
            if payload.new_controller_version != next {
                return Err(ContractError::InvalidControllerVersion);
            }
        }
    }
    Ok(())
}

pub(crate) fn create(
    env: &Env,
    proposer: Address,
    kind: ProposalKind,
) -> Result<u64, ContractError> {
    proposer.require_auth();
    let configured = storage::policy(env).ok_or(ContractError::InvalidPolicy)?;
    if !policy::is_approver(&configured, &proposer) {
        return Err(ContractError::NotApprover);
    }
    validate_kind(env, &kind)?;

    let current_ledger = env.ledger().sequence();
    let expires_ledger = current_ledger
        .checked_add(configured.proposal_lifetime_ledgers)
        .ok_or(ContractError::ArithmeticOverflow)?;
    let proposal_id = storage::nonce(env)
        .ok_or(ContractError::InvalidPolicy)?
        .checked_add(1)
        .ok_or(ContractError::ArithmeticOverflow)?;
    let governance_epoch = storage::epoch(env).ok_or(ContractError::InvalidPolicy)?;
    let proposal = Proposal {
        id: proposal_id,
        proposer: proposer.clone(),
        kind,
        governance_epoch,
        created_ledger: current_ledger,
        expires_ledger,
        approval_count: 0,
        approved_ledger: None,
        execute_after_ledger: None,
        status: StoredProposalStatus::Active,
    };
    storage::set_proposal(env, &proposal);
    let required_ttl = configured
        .proposal_lifetime_ledgers
        .checked_add(PROPOSAL_TTL_SAFETY_BUFFER)
        .ok_or(ContractError::TtlConfigurationInvalid)?;
    env.storage().persistent().extend_ttl_with_limits(
        &DataKey::Proposal(proposal_id),
        required_ttl,
        1,
        env.storage().max_ttl(),
    );
    storage::set_nonce(env, proposal_id);
    ProposalCreated {
        proposal_id,
        proposer,
        governance_epoch,
        expires_ledger,
    }
    .publish(env);
    Ok(proposal_id)
}
