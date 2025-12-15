use std::str::FromStr;

use crate::{
    tx_builders,
    types::{AaveArgs, Step},
    Contract, ContractExt,
};
use alloy_primitives::Address;
use near_sdk::{near, Promise};

#[near]
impl Contract {
    pub fn build_and_sign_aave_supply_tx(
        &mut self,
        args: AaveArgs,
        callback_gas_tgas: u64,
    ) -> Promise {
        self.assert_agent_is_calling();
        self.assert_step_is_next(Step::AaveSupply);

        let cfg = self.get_chain_config_from_step_and_current_session(Step::AaveSupply);

        let mut tx = args.clone().partial_transaction;
        tx.input = tx_builders::build_aave_supply_tx(args, cfg.aave.clone());
        tx.to = Some(
            Address::from_str(&cfg.aave.lending_pool_address)
                .expect("Invalid lending pool")
                .into_array(),
        );

        self.trigger_signature(Step::AaveSupply, tx, callback_gas_tgas)
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
