use std::str::FromStr;

use crate::{
    tx_builders,
    types::{CCTPMintArgs, Step},
    Contract, ContractExt,
};
use alloy_primitives::Address;
use near_sdk::{near, Promise};

#[near]
impl Contract {
    pub fn build_and_sign_cctp_mint_tx(
        &mut self,
        args: CCTPMintArgs,
        callback_gas_tgas: u64,
    ) -> Promise {
        self.assert_agent_is_calling();
        let cfg = self.get_chain_config_from_step_and_current_session(Step::CCTPMint);

        let mut tx = args.clone().partial_mint_transaction;
        tx.input = tx_builders::build_cctp_mint_tx(args);
        tx.to = Some(
            Address::from_str(&cfg.cctp.transmitter_address)
                .expect("Invalid transmitter")
                .into_array(),
        );

        self.trigger_signature(Step::CCTPMint, tx, callback_gas_tgas)
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
