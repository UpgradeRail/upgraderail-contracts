use soroban_sdk::{BytesN, Env};

use crate::errors::ContractError;
use crate::policy::PROPOSAL_TTL_SAFETY_BUFFER;
use crate::storage;
use crate::types::{DataKey, StoredProposalStatus};

// The Testnet settings query on 2026-10-01 returned max_entry_ttl=3,110,400
// and min_persistent_ttl=120,960. Stellar's storage guide also lists the
// 3,110,400-ledger maximum: https://developers.stellar.org/docs/build/guides/storage/storage-strategies
// Runtime max_ttl() keeps extensions within the active network setting.
pub const INSTANCE_TTL_TARGET: u32 = 2_073_600;
pub const PERSISTENT_RECORD_TTL_TARGET: u32 = 2_073_600;
pub const MIN_TTL_EXTENSION: u32 = 17_280;
pub const MAX_TTL_EXTENSION: u32 = 3_110_400;

fn target(env: &Env, preferred: u32) -> u32 {
    core::cmp::min(preferred, core::cmp::min(env.storage().max_ttl(), MAX_TTL_EXTENSION))
}

pub(crate) fn maintain_controller(env: &Env) -> Result<(), ContractError> {
    let extend_to = target(env, INSTANCE_TTL_TARGET);
    if extend_to == 0 {
        return Err(ContractError::TtlConfigurationInvalid);
    }
    env.storage().instance().extend_ttl_with_limits(
        extend_to,
        MIN_TTL_EXTENSION,
        MAX_TTL_EXTENSION,
    );
    Ok(())
}

pub(crate) fn maintain_fleet(env: &Env, id: &BytesN<32>) -> Result<(), ContractError> {
    let fleet = storage::fleet(env, id).ok_or(ContractError::FleetNotFound)?;
    if !storage::tag_used(env, &fleet.tag) {
        return Err(ContractError::StorageInvariantViolated);
    }
    if !env.executable_refs().has(&fleet.tag) {
        return Err(ContractError::ExecutableRefMissing);
    }
    let extend_to = target(env, PERSISTENT_RECORD_TTL_TARGET);
    if extend_to == 0 {
        return Err(ContractError::TtlConfigurationInvalid);
    }
    env.storage().persistent().extend_ttl_with_limits(
        &DataKey::Fleet(id.clone()), extend_to, MIN_TTL_EXTENSION, MAX_TTL_EXTENSION,
    );
    env.storage().persistent().extend_ttl_with_limits(
        &DataKey::FleetByTag(fleet.tag.clone()), extend_to, MIN_TTL_EXTENSION, MAX_TTL_EXTENSION,
    );
    env.executable_refs().extend_ttl_with_limits(
        &fleet.tag, extend_to, MIN_TTL_EXTENSION, MAX_TTL_EXTENSION,
    );
    Ok(())
}

pub(crate) fn maintain_proposal(env: &Env, id: u64) -> Result<(), ContractError> {
    let proposal = storage::proposal(env, id).ok_or(ContractError::ProposalNotFound)?;
    let now = env.ledger().sequence();
    let remaining = proposal.expires_ledger.saturating_sub(now);
    let expiry_target = remaining
        .checked_add(PROPOSAL_TTL_SAFETY_BUFFER)
        .ok_or(ContractError::TtlConfigurationInvalid)?;
    let preferred = core::cmp::max(PERSISTENT_RECORD_TTL_TARGET, expiry_target);
    let extend_to = target(env, preferred);
    if expiry_target > extend_to {
        return Err(ContractError::TtlConfigurationInvalid);
    }
    env.storage().persistent().extend_ttl_with_limits(
        &DataKey::Proposal(id), extend_to, MIN_TTL_EXTENSION, MAX_TTL_EXTENSION,
    );
    if proposal.status == StoredProposalStatus::Active
        && proposal.governance_epoch == storage::epoch(env).ok_or(ContractError::InvalidPolicy)?
    {
        let policy = storage::policy(env).ok_or(ContractError::InvalidPolicy)?;
        for approver in policy.approvers.iter() {
            if storage::has_approval(env, id, &approver) {
                env.storage().persistent().extend_ttl_with_limits(
                    &DataKey::Approval(id, approver), extend_to, MIN_TTL_EXTENSION, MAX_TTL_EXTENSION,
                );
            }
        }
    }
    Ok(())
}
