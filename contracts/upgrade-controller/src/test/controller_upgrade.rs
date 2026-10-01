use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    Address, BytesN, Env, Vec,
};

use crate::types::{GovernancePolicy, ProposalKind, ProposalState, UpgradeControllerProposal};
use crate::{UpgradeController, UpgradeControllerClient};

const CONTROLLER_WASM: &[u8] =
    include_bytes!("../../../../fixtures/wasm/upgrade_controller_v1.wasm");

#[test]
fn self_upgrade_is_governed_and_invalid_wasm_rolls_back() {
    let env = Env::default();
    env.mock_all_auths();
    let approver = Address::generate(&env);
    let mut approvers = Vec::new(&env);
    approvers.push_back(approver.clone());
    let policy = GovernancePolicy {
        approvers,
        threshold: 1,
        timelock_ledgers: 3,
        proposal_lifetime_ledgers: 100,
    };
    let controller = env.register(UpgradeController, (&policy,));
    let client = UpgradeControllerClient::new(&env, &controller);
    let real_hash = env.deployer().upload_contract_wasm(CONTROLLER_WASM);
    let manifest_hash = BytesN::from_array(&env, &[3; 32]);

    let mismatch = ProposalKind::UpgradeController(UpgradeControllerProposal {
        expected_controller_version: 2,
        new_controller_version: 3,
        new_wasm_hash: real_hash.clone(),
        manifest_hash: manifest_hash.clone(),
    });
    assert!(client.try_create_proposal(&approver, &mismatch).is_err());
    let nonsequential = ProposalKind::UpgradeController(UpgradeControllerProposal {
        expected_controller_version: 1,
        new_controller_version: 3,
        new_wasm_hash: real_hash.clone(),
        manifest_hash: manifest_hash.clone(),
    });
    assert!(client
        .try_create_proposal(&approver, &nonsequential)
        .is_err());

    let missing = ProposalKind::UpgradeController(UpgradeControllerProposal {
        expected_controller_version: 1,
        new_controller_version: 2,
        new_wasm_hash: BytesN::from_array(&env, &[8; 32]),
        manifest_hash: manifest_hash.clone(),
    });
    let missing_id = client.create_proposal(&approver, &missing);
    client.approve(&missing_id, &approver);
    env.ledger()
        .set_sequence_number(env.ledger().sequence() + 3);
    assert!(client.try_execute_proposal(&missing_id).is_err());
    assert_eq!(client.get_controller_version(), 1);
    assert_eq!(client.get_proposal_state(&missing_id), ProposalState::Ready);

    let valid = ProposalKind::UpgradeController(UpgradeControllerProposal {
        expected_controller_version: 1,
        new_controller_version: 2,
        new_wasm_hash: real_hash,
        manifest_hash,
    });
    let valid_id = client.create_proposal(&approver, &valid);
    client.approve(&valid_id, &approver);
    env.ledger()
        .set_sequence_number(env.ledger().sequence() + 3);
    client.execute_proposal(&valid_id);
    assert_eq!(client.get_controller_version(), 2);
    assert_eq!(
        client.get_proposal_state(&valid_id),
        ProposalState::Executed
    );
}
