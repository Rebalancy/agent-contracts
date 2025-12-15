use std::str::FromStr;

use crate::{
    tx_builders,
    types::{RebalancerArgs, Step},
    Contract, ContractExt,
};
use alloy_primitives::Address;
use near_sdk::{near, Promise};

#[near]
impl Contract {
    pub fn build_and_sign_withdraw_for_crosschain_allocation_tx(
        &mut self,
        rebalancer_args: RebalancerArgs,
        callback_gas_tgas: u64,
    ) -> Promise {
        self.assert_agent_is_calling();
        let cfg =
            self.get_chain_config_from_step_and_current_session(Step::RebalancerWithdrawToAllocate);

        let mut tx = rebalancer_args.clone().partial_transaction;
        tx.input = tx_builders::build_withdraw_for_crosschain_allocation_tx(rebalancer_args);
        tx.to = Some(
            Address::from_str(&cfg.rebalancer.vault_address)
                .expect("Invalid vault")
                .into_array(),
        );

        self.trigger_signature(Step::RebalancerWithdrawToAllocate, tx, callback_gas_tgas)
    }
}

#[cfg(test)]
mod maintests {
    use crate::test_helpers::*;
    use crate::types::*;
    use near_sdk::env;

    #[test]
    fn test_() {}
}
