use std::str::FromStr;

use crate::{
    constants::*, ecdsa, external::this_contract, tx_builders, types::ApproveCctpBurnArgs,
    Contract, ContractExt,
};
use alloy_primitives::Address;
use near_sdk::{env, near, Gas, Promise};

#[near]
impl Contract {
    pub fn build_and_sign_cctp_approve_burn_tx(
        &mut self,
        args: ApproveCctpBurnArgs,
        callback_gas_tgas: u64,
    ) -> Promise {
        self.assert_agent_is_calling();

        let config = self.get_chain_config(&args.chain_id);

        let mut tx = args.clone().partial_transaction;
        tx.input = tx_builders::build_cctp_approve_burn_tx(
            args.amount,
            config.cctp.messenger_address.clone(),
        );
        tx.to = Some(
            Address::from_str(&config.cctp.usdc_address)
                .expect("Invalid USDC address")
                .into_array(),
        );

        let payload_hash = self.hash_payload(&tx);

        ecdsa::get_sig(payload_hash, PATH.to_string(), KEY_VERSION).then(
            this_contract::ext(env::current_account_id())
                .with_static_gas(Gas::from_tgas(callback_gas_tgas))
                .sign_generic_callback(tx),
        )
    }
}

#[cfg(test)]
mod maintests {
    use crate::test_helpers::*;
    use crate::types::*;
    use near_sdk::env;
}
