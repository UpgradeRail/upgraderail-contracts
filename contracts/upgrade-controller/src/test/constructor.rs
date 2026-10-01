use soroban_sdk::{testutils::Address as _, Address, ContractExecutable, Env, Vec};

use crate::storage;
use crate::types::GovernancePolicy;
use crate::{UpgradeController, UpgradeControllerClient};

fn policy(env: &Env) -> GovernancePolicy {
    let mut approvers = Vec::new(env);
    approvers.push_back(Address::generate(env));
    GovernancePolicy {
        approvers,
        threshold: 1,
        timelock_ledgers: 5,
        proposal_lifetime_ledgers: 100,
    }
}

#[test]
fn constructor_stores_governance_state() {
    let env = Env::default();
    let configured = policy(&env);
    let id = env.register(UpgradeController, (&configured,));
    let client = UpgradeControllerClient::new(&env, &id);

    assert_eq!(client.get_policy(), configured);
    assert_eq!(client.get_governance_epoch(), 1);
    assert_eq!(client.get_controller_version(), 1);
    assert_eq!(env.as_contract(&id, || storage::nonce(&env)), Some(0));
}

#[test]
#[should_panic]
fn constructor_rejects_invalid_policy() {
    let env = Env::default();
    let mut configured = policy(&env);
    configured.threshold = 0;
    env.register(UpgradeController, (&configured,));
}

#[test]
fn uploaded_wasm_deployment_runs_constructor() {
    let env = Env::default();
    let configured = policy(&env);
    let host = env.register(UpgradeController, (&configured,));
    let wasm: &[u8] = include_bytes!("../../../../fixtures/wasm/upgrade_controller_v1.wasm");
    let hash = env.deployer().upload_contract_wasm(wasm);
    let deployed = env.as_contract(&host, || {
        env.deployer()
            .with_current_contract([6; 32])
            .deploy_contract(ContractExecutable::Wasm(hash), (&configured,))
    });
    let client = UpgradeControllerClient::new(&env, &deployed);
    assert_eq!(client.get_policy(), configured);
    assert_eq!(client.get_governance_epoch(), 1);
    assert_eq!(client.get_controller_version(), 1);
}
