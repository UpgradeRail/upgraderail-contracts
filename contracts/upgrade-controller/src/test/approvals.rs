use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String, Vec};

use crate::storage;
use crate::types::{CreateFleetProposal, GovernancePolicy, ProposalKind};
use crate::{UpgradeController, UpgradeControllerClient};

#[test]
fn threshold_starts_once_and_duplicate_approval_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let first = Address::generate(&env);
    let second = Address::generate(&env);
    let third = Address::generate(&env);
    let outsider = Address::generate(&env);
    let mut approvers = Vec::new(&env);
    approvers.push_back(first.clone());
    approvers.push_back(second.clone());
    approvers.push_back(third.clone());
    let policy = GovernancePolicy {
        approvers,
        threshold: 2,
        timelock_ledgers: 5,
        proposal_lifetime_ledgers: 100,
    };
    let contract = env.register(UpgradeController, (&policy,));
    let client = UpgradeControllerClient::new(&env, &contract);
    let kind = ProposalKind::CreateFleet(CreateFleetProposal {
        fleet_id: BytesN::from_array(&env, &[1; 32]),
        tag: String::from_str(&env, "fleet-a"),
        initial_wasm_hash: BytesN::from_array(&env, &[2; 32]),
        manifest_hash: BytesN::from_array(&env, &[3; 32]),
    });
    let id = client.create_proposal(&first, &kind);

    assert!(client.try_approve(&id, &outsider).is_err());
    client.approve(&id, &first);
    assert!(client.try_approve(&id, &first).is_err());
    let pending = env.as_contract(&contract, || storage::proposal(&env, id).unwrap());
    assert_eq!(pending.approval_count, 1);
    assert_eq!(pending.approved_ledger, None);
    assert_eq!(pending.execute_after_ledger, None);

    client.approve(&id, &second);
    let threshold = env.as_contract(&contract, || storage::proposal(&env, id).unwrap());
    assert_eq!(threshold.approval_count, 2);
    assert_eq!(threshold.approved_ledger, Some(env.ledger().sequence()));
    assert_eq!(
        threshold.execute_after_ledger,
        Some(env.ledger().sequence() + 5)
    );
    client.approve(&id, &third);
    let after = env.as_contract(&contract, || storage::proposal(&env, id).unwrap());
    assert_eq!(after.approval_count, 3);
    assert_eq!(after.approved_ledger, threshold.approved_ledger);
    assert_eq!(after.execute_after_ledger, threshold.execute_after_ledger);
}
