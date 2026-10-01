use soroban_sdk::{contractevent, Address};

#[contractevent]
pub struct ProposalCreated {
    #[topic]
    pub proposal_id: u64,
    pub proposer: Address,
    pub governance_epoch: u64,
    pub expires_ledger: u32,
}
