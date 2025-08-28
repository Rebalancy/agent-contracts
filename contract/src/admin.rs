use near_sdk::{near, require};

use crate::types::ChainConfig;
use crate::{Contract, ContractExt};

#[near]
impl Contract {
    pub fn add_supported_chain(&mut self, new_chain: ChainConfig) {
        self.require_owner();
        require!(
            !self.supported_chains.contains(&new_chain.chain_id),
            "Chain already supported"
        );
        self.supported_chains.push(new_chain.chain_id.clone());
        self.config.insert(new_chain.chain_id, new_chain.config);
    }

    pub fn approve_codehash(&mut self, codehash: String) {
        self.require_owner();
        self.approved_codehashes.insert(codehash);
    }
}
