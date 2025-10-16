use near_sdk::{env, require};
use omni_transaction::evm::EVMTransaction;

use crate::{
    types::{CacheKey, ChainId, Config, Flow, PayloadType, Step},
    Contract,
};

impl Contract {
    pub(crate) fn hash_payload(&self, ethereum_tx: &EVMTransaction) -> [u8; 32] {
        env::keccak256(&ethereum_tx.build_for_signing())
            .try_into()
            .expect("Payload must be 32 bytes long")
    }

    fn get_chain_id_from_the_step(&self, step: Step) -> ChainId {
        let log = self.get_activity_log();
        let flow = self.get_active_session().flow.clone();

        match (flow, step) {
            // -------- Aave -> Aave --------
            (Flow::AaveToAave, PayloadType::AaveWithdraw)
            | (Flow::AaveToAave, PayloadType::CCTPBurn) => log.source_chain,

            (Flow::AaveToAave, PayloadType::CCTPMint)
            | (Flow::AaveToAave, PayloadType::AaveSupply) => log.destination_chain,

            // -------- Rebalancer -> Aave --------
            (Flow::RebalancerToAave, PayloadType::RebalancerWithdrawToAllocate)
            | (Flow::RebalancerToAave, PayloadType::CCTPBurn) => log.source_chain,

            (Flow::RebalancerToAave, PayloadType::CCTPMint)
            | (Flow::RebalancerToAave, PayloadType::AaveSupply) => log.destination_chain,

            // -------- Aave -> Rebalancer --------
            (Flow::AaveToRebalancer, PayloadType::AaveWithdraw)
            | (Flow::AaveToRebalancer, PayloadType::CCTPBurn) => log.source_chain,

            (Flow::AaveToRebalancer, PayloadType::CCTPMint)
            | (Flow::AaveToRebalancer, PayloadType::RebalancerDeposit) => log.destination_chain,

            _ => env::panic_str("Invalid (flow, step) combination for chain selection"),
        }
    }

    pub(crate) fn get_chain_config_for_step(&self, step: Step) -> &Config {
        let chain_id = self.get_chain_id_from_the_step(step);
        self.get_chain_config(&chain_id)
    }

    pub(crate) fn is_chain_supported(&self, chain_id: &ChainId) {
        require!(
            self.supported_chains.contains(chain_id),
            "Chain not supported"
        );
    }

    pub fn has_signature(&self, step: Step) -> bool {
        let nonce = self.get_active_session().nonce;
        self.signatures_by_nonce_and_type
            .contains_key(&CacheKey::new(nonce, step as u8))
    }

    pub(crate) fn assert_step_is_next(&self, requested: Step) {
        for &st in self.get_active_session().flow.sequence() {
            if !self.has_signature(st) {
                require!(st == requested, "Wrong step for current position");
                return;
            }
        }
        env::panic_str("Flow already finished");
    }

    pub(crate) fn assert_no_active_session(&mut self) {
        require!(self.active_session.is_none(), "Another action in progress");
    }
}
