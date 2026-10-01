#![no_std]

use soroban_sdk::{
    contract, contractimpl, Address, BytesN, ContractExecutable, ContractExecutableRef, Env, String,
};

#[contract]
pub struct ExternalRefFactory;

#[contractimpl]
impl ExternalRefFactory {
    pub fn deploy(env: Env, owner: Address, tag: String, salt: BytesN<32>) -> Address {
        env.deployer()
            .with_current_contract(salt.to_array())
            .deploy_contract(
                ContractExecutable::ExternalRef(ContractExecutableRef { owner, tag }),
                (),
            )
    }
}
