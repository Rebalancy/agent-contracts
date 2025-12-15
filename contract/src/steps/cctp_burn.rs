use crate::{tx_builders, types::*, Contract, ContractExt};
use alloy_primitives::Address;
use near_sdk::{near, Promise};
use std::str::FromStr;

#[near]
impl Contract {
    pub fn build_and_sign_cctp_burn_tx(
        &mut self,
        args: CCTPBurnArgs,
        callback_gas_tgas: u64,
    ) -> Promise {
        self.assert_agent_is_calling();
        let cfg = self.get_chain_config_from_step_and_current_session(Step::CCTPBurn);

        let mut tx = args.clone().partial_burn_transaction;
        tx.input = tx_builders::build_cctp_burn_tx(args);
        tx.to = Some(
            Address::from_str(&cfg.cctp.messenger_address)
                .expect("Invalid messenger")
                .into_array(),
        );

        self.trigger_signature(Step::CCTPBurn, tx, callback_gas_tgas)
    }
}

#[cfg(test)]
mod maintests {
    use crate::test_helpers::*;
    use crate::types::*;
    use near_sdk::env;

    #[test]
    fn test_build_and_sign_cctp_burn_tx() {}
}
