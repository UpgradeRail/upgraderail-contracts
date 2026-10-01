use soroban_sdk::{Address, Env};

use crate::errors::ContractError;
use crate::events::{ApprovalRevoked, ProposalApproved, ThresholdReached, ThresholdReset};
use crate::policy::{self, PROPOSAL_TTL_SAFETY_BUFFER};
use crate::storage;
use crate::types::{DataKey, Proposal, StoredProposalStatus};

pub(crate) fn active(env: &Env, id: u64, check_expiry: bool) -> Result<Proposal, ContractError> {
    let proposal = storage::proposal(env, id).ok_or(ContractError::ProposalNotFound)?;
    if proposal.status != StoredProposalStatus::Active {
        return Err(ContractError::ProposalNotActive);
    }
    let epoch = storage::epoch(env).ok_or(ContractError::InvalidPolicy)?;
    if proposal.governance_epoch != epoch {
        return Err(ContractError::ProposalStale);
    }
    if check_expiry && env.ledger().sequence() >= proposal.expires_ledger {
        return Err(ContractError::ProposalExpired);
    }
    Ok(proposal)
}

pub(crate) fn approve(env: &Env, id: u64, approver: Address) -> Result<(), ContractError> {
    approver.require_auth();
    let configured = storage::policy(env).ok_or(ContractError::InvalidPolicy)?;
    if !policy::is_approver(&configured, &approver) {
        return Err(ContractError::NotApprover);
    }
    let mut proposal = active(env, id, true)?;
    if storage::has_approval(env, id, &approver) {
        return Err(ContractError::AlreadyApproved);
    }
    let next_count = proposal
        .approval_count
        .checked_add(1)
        .ok_or(ContractError::ArithmeticOverflow)?;
    if next_count > configured.approvers.len() {
        return Err(ContractError::ArithmeticOverflow);
    }
    let now = env.ledger().sequence();
    if next_count == configured.threshold {
        let execute_after = now
            .checked_add(configured.timelock_ledgers)
            .ok_or(ContractError::ArithmeticOverflow)?;
        proposal.approved_ledger = Some(now);
        proposal.execute_after_ledger = Some(execute_after);
        ThresholdReached {
            proposal_id: id,
            approved_ledger: now,
            execute_after_ledger: execute_after,
        }
        .publish(env);
    }
    proposal.approval_count = next_count;
    storage::set_approval(env, id, &approver);
    let remaining = proposal
        .expires_ledger
        .checked_sub(now)
        .ok_or(ContractError::ProposalExpired)?;
    let required_ttl = remaining
        .checked_add(PROPOSAL_TTL_SAFETY_BUFFER)
        .ok_or(ContractError::TtlConfigurationInvalid)?;
    env.storage().persistent().extend_ttl_with_limits(
        &DataKey::Approval(id, approver.clone()),
        required_ttl,
        1,
        env.storage().max_ttl(),
    );
    storage::set_proposal(env, &proposal);
    ProposalApproved {
        proposal_id: id,
        approver,
        approval_count: next_count,
    }
    .publish(env);
    Ok(())
}

pub(crate) fn revoke(env: &Env, id: u64, approver: Address) -> Result<(), ContractError> {
    approver.require_auth();
    let configured = storage::policy(env).ok_or(ContractError::InvalidPolicy)?;
    let mut proposal = active(env, id, false)?;
    if !storage::has_approval(env, id, &approver) {
        return Err(ContractError::ApprovalNotFound);
    }
    let previous_count = proposal.approval_count;
    let next_count = previous_count
        .checked_sub(1)
        .ok_or(ContractError::ArithmeticOverflow)?;
    proposal.approval_count = next_count;
    if previous_count >= configured.threshold && next_count < configured.threshold {
        proposal.approved_ledger = None;
        proposal.execute_after_ledger = None;
        ThresholdReset { proposal_id: id }.publish(env);
    }
    storage::remove_approval(env, id, &approver);
    storage::set_proposal(env, &proposal);
    ApprovalRevoked {
        proposal_id: id,
        approver,
        approval_count: next_count,
    }
    .publish(env);
    Ok(())
}
