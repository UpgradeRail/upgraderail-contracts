use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    Address, BytesN, Env, String, Vec,
};

use crate::types::{CreateFleetProposal, GovernancePolicy, ProposalKind, ProposalState};
use crate::{UpgradeController, UpgradeControllerClient};

const V1_WASM: &[u8] = include_bytes!("../../../../fixtures/wasm/fleet_v1.wasm");

fn setup() -> (Env, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let first = Address::generate(&env);
    let second = Address::generate(&env);
    let mut approvers = Vec::new(&env);
    approvers.push_back(first.clone());
    approvers.push_back(second.clone());
    let policy = GovernancePolicy {
        approvers,
        threshold: 2,
        timelock_ledgers: 3,
        proposal_lifetime_ledgers: 100,
    };
    let controller = env.register(UpgradeController, (&policy,));
    (env, controller, first, second)
}

#[test]
fn missing_uploaded_wasm_rolls_back_fleet_creation() {
    let (env, controller, first, second) = setup();
    let client = UpgradeControllerClient::new(&env, &controller);
    let fleet_id = BytesN::from_array(&env, &[5; 32]);
    let kind = ProposalKind::CreateFleet(CreateFleetProposal {
        fleet_id: fleet_id.clone(),
        tag: String::from_str(&env, "missing-wasm"),
        initial_wasm_hash: BytesN::from_array(&env, &[8; 32]),
        manifest_hash: BytesN::from_array(&env, &[3; 32]),
    });
    let id = client.create_proposal(&first, &kind);
    client.approve(&id, &first);
    client.approve(&id, &second);
    env.ledger()
        .set_sequence_number(env.ledger().sequence() + 3);
    assert!(client.try_execute_proposal(&id).is_err());
    assert!(client.try_get_fleet(&fleet_id).is_err());
    assert_eq!(client.get_proposal_state(&id), ProposalState::Ready);
}

#[test]
fn existing_executable_reference_blocks_fleet_creation() {
    let (env, controller, first, _) = setup();
    let client = UpgradeControllerClient::new(&env, &controller);
    let initial = env.deployer().upload_contract_wasm(V1_WASM);
    let tag = String::from_str(&env, "reserved-fleet");
    env.as_contract(&controller, || env.executable_refs().set(&tag, &initial));

    let kind = ProposalKind::CreateFleet(CreateFleetProposal {
        fleet_id: BytesN::from_array(&env, &[8; 32]),
        tag,
        initial_wasm_hash: initial,
        manifest_hash: BytesN::from_array(&env, &[3; 32]),
    });
    assert!(client.try_create_proposal(&first, &kind).is_err());
}
