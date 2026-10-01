use soroban_sdk::{testutils::Address as _, Address, Env, Vec};

use crate::errors::ContractError;
use crate::policy::{validate, MAX_APPROVERS, PROPOSAL_TTL_SAFETY_BUFFER};
use crate::types::{GovernancePolicy, ProposalKind, UpdatePolicyProposal};
use crate::UpgradeControllerClient;

fn policy(env: &Env) -> GovernancePolicy {
    let mut approvers = Vec::new(env);
    approvers.push_back(Address::generate(env));
    approvers.push_back(Address::generate(env));
    GovernancePolicy {
        approvers,
        threshold: 2,
        timelock_ledgers: 10,
        proposal_lifetime_ledgers: 100,
    }
}

#[test]
fn accepts_valid_policy_and_bounded_approver_count() {
    let env = Env::default();
    let mut configured = policy(&env);
    assert_eq!(validate(&env, &configured), Ok(()));

    configured.approvers = Vec::new(&env);
    for _ in 0..MAX_APPROVERS {
        configured.approvers.push_back(Address::generate(&env));
    }
    assert_eq!(validate(&env, &configured), Ok(()));
    configured.approvers.push_back(Address::generate(&env));
    assert_eq!(
        validate(&env, &configured),
        Err(ContractError::TooManyApprovers)
    );
}

#[test]
fn rejects_invalid_approvers_and_threshold() {
    let env = Env::default();
    let mut configured = policy(&env);
    configured.approvers = Vec::new(&env);
    assert_eq!(validate(&env, &configured), Err(ContractError::NoApprovers));

    configured = policy(&env);
    configured
        .approvers
        .push_back(configured.approvers.get(0).unwrap());
    assert_eq!(
        validate(&env, &configured),
        Err(ContractError::DuplicateApprover)
    );

    configured = policy(&env);
    configured.threshold = 0;
    assert_eq!(
        validate(&env, &configured),
        Err(ContractError::InvalidThreshold)
    );
    configured.threshold = 3;
    assert_eq!(
        validate(&env, &configured),
        Err(ContractError::InvalidThreshold)
    );
}

#[test]
fn rejects_invalid_timelock_and_lifetime() {
    let env = Env::default();
    let mut configured = policy(&env);
    configured.timelock_ledgers = 0;
    assert_eq!(
        validate(&env, &configured),
        Err(ContractError::InvalidTimelock)
    );

    configured.timelock_ledgers = 10;
    configured.proposal_lifetime_ledgers = 10;
    assert_eq!(
        validate(&env, &configured),
        Err(ContractError::InvalidProposalLifetime)
    );

    configured.proposal_lifetime_ledgers =
        core::cmp::min(env.storage().max_ttl(), crate::ttl::MAX_TTL_EXTENSION)
            - PROPOSAL_TTL_SAFETY_BUFFER;
    assert_eq!(validate(&env, &configured), Ok(()));
    configured.proposal_lifetime_ledgers += 1;
    assert_eq!(
        validate(&env, &configured),
        Err(ContractError::InvalidProposalLifetime)
    );
}

#[test]
fn twenty_approvers_can_complete_threshold_under_test_budget() {
    let env = Env::default();
    env.mock_all_auths();
    let mut configured = policy(&env);
    configured.approvers = Vec::new(&env);
    for _ in 0..MAX_APPROVERS {
        configured.approvers.push_back(Address::generate(&env));
    }
    configured.threshold = MAX_APPROVERS;
    let wasm: &[u8] = include_bytes!("../../../../fixtures/wasm/upgrade_controller_v1.wasm");
    let contract = env.register(wasm, (&configured,));
    let client = UpgradeControllerClient::new(&env, &contract);
    let first = configured.approvers.get(0).unwrap();
    let id = client.create_proposal(
        &first,
        &ProposalKind::UpdatePolicy(UpdatePolicyProposal {
            policy: configured.clone(),
        }),
    );
    for approver in configured.approvers.iter() {
        env.cost_estimate().budget().reset_tracker();
        client.approve(&id, &approver);
        assert!(env.cost_estimate().budget().cpu_instruction_cost() < 400_000_000);
        assert!(env.cost_estimate().budget().memory_bytes_cost() < 41_943_040);
    }
    assert_eq!(client.get_proposal(&id).approval_count, MAX_APPROVERS);
}
