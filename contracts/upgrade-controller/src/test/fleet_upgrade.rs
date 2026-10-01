use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Ledger as _},
    vec, Address, BytesN, ContractExecutable, ContractExecutableRef, Env, IntoVal, String, Symbol,
    Vec,
};

use crate::storage;
use crate::types::{
    CreateFleetProposal, Fleet, GovernancePolicy, ProposalKind, UpgradeFleetProposal,
};
use crate::{UpgradeController, UpgradeControllerClient};

const V1_WASM: &[u8] = include_bytes!("../../../../fixtures/wasm/fleet_v1.wasm");
const V2_WASM: &[u8] = include_bytes!("../../../../fixtures/wasm/fleet_v2_compatible.wasm");
const MIGRATION_WASM: &[u8] = include_bytes!("../../../../fixtures/wasm/fleet_v2_migration.wasm");

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

fn approve_and_execute(
    env: &Env,
    client: &UpgradeControllerClient,
    id: u64,
    first: &Address,
    second: &Address,
) {
    client.approve(&id, first);
    client.approve(&id, second);
    assert!(client.try_execute_proposal(&id).is_err());
    env.ledger()
        .set_sequence_number(env.ledger().sequence() + 3);
    client.execute_proposal(&id);
}

#[test]
fn uploaded_wasm_creates_reference_and_upgrades_two_instances() {
    let (env, controller, first, second) = setup();
    let client = UpgradeControllerClient::new(&env, &controller);
    let initial = env.deployer().upload_contract_wasm(V1_WASM);
    let next = env.deployer().upload_contract_wasm(V2_WASM);
    let fleet_id = BytesN::from_array(&env, &[1; 32]);
    let tag = String::from_str(&env, "shared-fleet");
    let create = ProposalKind::CreateFleet(CreateFleetProposal {
        fleet_id: fleet_id.clone(),
        tag: tag.clone(),
        initial_wasm_hash: initial.clone(),
        manifest_hash: BytesN::from_array(&env, &[3; 32]),
    });
    let create_id = client.create_proposal(&first, &create);
    approve_and_execute(&env, &client, create_id, &first, &second);
    let fleet = client.get_fleet(&fleet_id);
    assert_eq!(fleet.id, fleet_id);
    assert_eq!(fleet.tag, tag);
    assert_eq!(fleet.created_ledger, env.ledger().sequence());
    assert_eq!(client.get_current_wasm(&fleet_id), initial);
    assert!(client.try_create_proposal(&first, &create).is_err());
    let reused_tag = ProposalKind::CreateFleet(CreateFleetProposal {
        fleet_id: BytesN::from_array(&env, &[6; 32]),
        tag: tag.clone(),
        initial_wasm_hash: initial.clone(),
        manifest_hash: BytesN::from_array(&env, &[3; 32]),
    });
    assert!(client.try_create_proposal(&first, &reused_tag).is_err());

    let deploy = |salt: [u8; 32]| {
        env.as_contract(&controller, || {
            env.deployer().with_current_contract(salt).deploy_contract(
                ContractExecutable::ExternalRef(ContractExecutableRef {
                    owner: controller.clone(),
                    tag: tag.clone(),
                }),
                (),
            )
        })
    };
    let instance_a = deploy([1; 32]);
    let instance_b = deploy([2; 32]);
    assert_eq!(
        env.invoke_contract::<u32>(&instance_a, &symbol_short!("version"), vec![&env]),
        1
    );
    assert_eq!(
        env.invoke_contract::<u32>(&instance_b, &symbol_short!("version"), vec![&env]),
        1
    );
    env.invoke_contract::<()>(
        &instance_a,
        &symbol_short!("set_value"),
        vec![&env, 7_i64.into_val(&env)],
    );
    env.invoke_contract::<()>(
        &instance_b,
        &symbol_short!("set_value"),
        vec![&env, 9_i64.into_val(&env)],
    );

    let upgrade = ProposalKind::UpgradeFleet(UpgradeFleetProposal {
        fleet_id: fleet_id.clone(),
        expected_wasm_hash: initial,
        new_wasm_hash: next.clone(),
        manifest_hash: BytesN::from_array(&env, &[4; 32]),
    });
    let upgrade_id = client.create_proposal(&first, &upgrade);
    let outdated_id = client.create_proposal(&first, &upgrade);
    approve_and_execute(&env, &client, upgrade_id, &first, &second);
    assert_eq!(client.get_current_wasm(&fleet_id), next);
    client.approve(&outdated_id, &first);
    client.approve(&outdated_id, &second);
    env.ledger()
        .set_sequence_number(env.ledger().sequence() + 3);
    assert!(client.try_execute_proposal(&outdated_id).is_err());
    for (instance, expected_value) in [(instance_a, 7_i64), (instance_b, 9_i64)] {
        assert_eq!(
            env.invoke_contract::<u32>(&instance, &symbol_short!("version"), vec![&env]),
            2
        );
        assert_eq!(
            env.invoke_contract::<i64>(&instance, &symbol_short!("get_value"), vec![&env]),
            expected_value
        );
        assert!(env.invoke_contract::<bool>(
            &instance,
            &Symbol::new(&env, "is_positive"),
            vec![&env]
        ));
    }
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
    assert_eq!(
        client.get_proposal_state(&id),
        crate::types::ProposalState::Ready
    );
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

#[test]
fn upgrade_rejects_current_candidate_and_missing_reference() {
    let (env, controller, first, second) = setup();
    let client = UpgradeControllerClient::new(&env, &controller);
    let initial = env.deployer().upload_contract_wasm(V1_WASM);
    let fleet_id = BytesN::from_array(&env, &[9; 32]);
    let tag = String::from_str(&env, "same-wasm-fleet");
    let create = ProposalKind::CreateFleet(CreateFleetProposal {
        fleet_id: fleet_id.clone(),
        tag,
        initial_wasm_hash: initial.clone(),
        manifest_hash: BytesN::from_array(&env, &[3; 32]),
    });
    let create_id = client.create_proposal(&first, &create);
    approve_and_execute(&env, &client, create_id, &first, &second);

    let unchanged = ProposalKind::UpgradeFleet(UpgradeFleetProposal {
        fleet_id,
        expected_wasm_hash: initial.clone(),
        new_wasm_hash: initial.clone(),
        manifest_hash: BytesN::from_array(&env, &[4; 32]),
    });
    assert!(client.try_create_proposal(&first, &unchanged).is_err());

    let orphan_id = BytesN::from_array(&env, &[10; 32]);
    env.as_contract(&controller, || {
        storage::set_fleet(
            &env,
            &Fleet {
                id: orphan_id.clone(),
                tag: String::from_str(&env, "orphan-fleet"),
                created_ledger: env.ledger().sequence(),
            },
        );
    });
    let orphan_upgrade = ProposalKind::UpgradeFleet(UpgradeFleetProposal {
        fleet_id: orphan_id,
        expected_wasm_hash: initial,
        new_wasm_hash: BytesN::from_array(&env, &[5; 32]),
        manifest_hash: BytesN::from_array(&env, &[4; 32]),
    });
    assert!(client.try_create_proposal(&first, &orphan_upgrade).is_err());
}

#[test]
fn migration_fixture_moves_existing_instance_state() {
    let (env, controller, first, second) = setup();
    let client = UpgradeControllerClient::new(&env, &controller);
    let initial = env.deployer().upload_contract_wasm(V1_WASM);
    let migration = env.deployer().upload_contract_wasm(MIGRATION_WASM);
    let fleet_id = BytesN::from_array(&env, &[7; 32]);
    let tag = String::from_str(&env, "migration-fleet");
    let id = client.create_proposal(
        &first,
        &ProposalKind::CreateFleet(CreateFleetProposal {
            fleet_id: fleet_id.clone(),
            tag: tag.clone(),
            initial_wasm_hash: initial.clone(),
            manifest_hash: BytesN::from_array(&env, &[3; 32]),
        }),
    );
    approve_and_execute(&env, &client, id, &first, &second);
    let instance = env.as_contract(&controller, || {
        env.deployer()
            .with_current_contract([7; 32])
            .deploy_contract(
                ContractExecutable::ExternalRef(ContractExecutableRef {
                    owner: controller.clone(),
                    tag,
                }),
                (),
            )
    });
    env.invoke_contract::<()>(
        &instance,
        &symbol_short!("set_value"),
        vec![&env, 42_i64.into_val(&env)],
    );
    let upgrade = client.create_proposal(
        &first,
        &ProposalKind::UpgradeFleet(UpgradeFleetProposal {
            fleet_id,
            expected_wasm_hash: initial,
            new_wasm_hash: migration,
            manifest_hash: BytesN::from_array(&env, &[4; 32]),
        }),
    );
    approve_and_execute(&env, &client, upgrade, &first, &second);
    assert_eq!(
        env.invoke_contract::<Option<i64>>(&instance, &symbol_short!("get_value"), vec![&env]),
        None
    );
    assert!(env.invoke_contract::<bool>(&instance, &symbol_short!("migrate"), vec![&env]));
    assert_eq!(
        env.invoke_contract::<Option<i64>>(&instance, &symbol_short!("get_value"), vec![&env]),
        Some(42)
    );
    assert_eq!(
        env.invoke_contract::<Option<u32>>(
            &instance,
            &Symbol::new(&env, "migration_ledger"),
            vec![&env],
        ),
        Some(env.ledger().sequence())
    );
    assert!(!env.invoke_contract::<bool>(&instance, &symbol_short!("migrate"), vec![&env]));
}
