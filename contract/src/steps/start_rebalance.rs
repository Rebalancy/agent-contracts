use crate::{types::*, Contract, ContractExt};
use near_sdk::{env, near};

#[near]
impl Contract {
    pub fn start_rebalance(
        &mut self,
        flow: Flow,
        source_chain: ChainId,
        destination_chain: ChainId,
        amount: u128,
    ) -> u64 {
        self.assert_no_active_session();
        self.assert_agent_is_calling();
        self.is_chain_supported(&source_chain);
        self.is_chain_supported(&destination_chain);

        let nonce = self.logs_nonce;
        self.logs_nonce += 1;

        self.logs.insert(
            nonce,
            ActivityLog {
                activity_type: AgentActionType::Rebalance,
                source_chain: source_chain,
                destination_chain,
                transactions: vec![],
                timestamp: env::block_timestamp_ms(),
                nonce,
                amount,
            },
        );

        self.active_session = Some(ActiveSession {
            nonce,
            flow,
            started_at: env::block_timestamp_ms(),
        });

        nonce
    }
}

#[cfg(test)]
mod maintests {
    use crate::test_helpers::*;
    use crate::types::*;
    use near_sdk::env;

    #[test]
    fn test_start_rebalance() {
        let mut contract = init_contract_with_defaults();

        let flow = Flow::RebalancerToAave;
        let source_chain = DEFAULT_SOURCE_CHAIN;
        let destination_chain = DEFAULT_DESTINATION_CHAIN;
        let amount: u128 = 1_000_000_000;

        let current_nonce =
            contract.start_rebalance(flow.clone(), source_chain, destination_chain, amount);

        assert_eq!(current_nonce, 0);
        assert!(contract.logs_nonce == 1);
        assert!(contract.logs.contains_key(&0));
        assert!(contract.active_session.is_some());

        let session = contract.active_session.as_ref().unwrap();
        assert_eq!(session.flow, flow);
        assert_eq!(session.nonce, 0);
        assert_eq!(session.started_at, env::block_timestamp_ms());

        let empty_vector: Vec<Vec<u8>> = vec![];

        let log = contract.logs.get(&0).unwrap();
        assert_eq!(log.activity_type, AgentActionType::Rebalance);
        assert_eq!(log.source_chain, source_chain);
        assert_eq!(log.destination_chain, destination_chain);
        assert_eq!(log.transactions, empty_vector);
        assert_eq!(log.timestamp, env::block_timestamp_ms());
        assert_eq!(log.nonce, 0);
        assert_eq!(log.amount, amount);
    }
}
