use std::str::FromStr;

use crate::{
    constants::*, ecdsa, external::this_contract, tx_builders, types::ApproveAaveSupplyArgs,
    Contract, ContractExt,
};
use alloy_primitives::Address;
use near_sdk::{env, near, Gas, Promise};

#[near]
impl Contract {
    pub fn build_and_sign_aave_approve_supply_tx(
        &mut self,
        args: ApproveAaveSupplyArgs,
        callback_gas_tgas: u64,
    ) -> Promise {
        self.assert_agent_is_calling();

        assert!(args.chain_id != self.source_chain); // @dev since Aave interaction in the source chain is via the Vault contract

        let config = self.get_chain_config(&args.chain_id);

        let mut tx = args.clone().partial_transaction;
        tx.input = tx_builders::build_aave_approve_supply_tx(
            args.amount,
            config.aave.lending_pool_address.clone(),
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
    use near_sdk::mock::MockAction;
    use near_sdk::test_utils::get_created_receipts;
    use near_sdk::Gas;
    use omni_transaction::evm::EVMTransaction;

    const DEFAULT_TGAS: u64 = 10;
    const DEFAULT_AMOUNT: u128 = 1_000_000_000u128;

    // TODO: Restore once access control is working
    // #[test]
    // #[should_panic]
    // fn fails_if_not_agent() {
    //     set_context(OWNER);

    //     let mut contract = init_contract_with_defaults();

    //     // No-agent
    //     set_context("random.near");

    //     let args = build_args();
    //     contract.build_and_sign_aave_approve_supply_tx(args, DEFAULT_TGAS);
    // }

    #[test]
    #[should_panic]
    fn fails_if_chain_is_source_chain() {
        set_context(OWNER);

        let mut contract = init_contract_with_defaults();

        let mut args = build_args();
        args.chain_id = contract.source_chain; // @dev this should fail

        contract.build_and_sign_aave_approve_supply_tx(args, DEFAULT_TGAS);
    }

    #[test]
    fn test_build_and_sign_aave_approve_supply_tx() {
        let mut contract = init_contract_with_defaults();

        let args = build_args();

        contract.build_and_sign_aave_approve_supply_tx(args, DEFAULT_TGAS);

        let receipts = get_created_receipts();
        assert!(
            receipts.len() >= 2,
            "Not enough receipts, received: {}",
            receipts.len()
        );

        let mut found = false;

        for r in receipts {
            for a in r.actions {
                match a {
                    MockAction::FunctionCallWeight {
                        method_name,
                        args: _,
                        attached_deposit,
                        prepaid_gas,
                        gas_weight: _,
                        ..
                    } => {
                        let method =
                            String::from_utf8(method_name).expect("method_name is not utf8");

                        if method == "sign_generic_callback" {
                            found = true;

                            // valid asserts
                            assert!(prepaid_gas > Gas::from_tgas(0));
                            assert!(attached_deposit.is_zero());

                            // TODO: continue asserts
                        }
                    }
                    _ => {}
                }
            }
        }

        assert!(found, "sign_generic_callback not found");
    }

    fn build_partial_tx() -> EVMTransaction {
        EVMTransaction {
            chain_id: 1,
            nonce: 1,
            to: None,
            input: vec![],
            value: 0,
            gas_limit: 100,
            max_fee_per_gas: 100,
            max_priority_fee_per_gas: 100,
            access_list: vec![],
        }
    }

    fn build_args() -> ApproveAaveSupplyArgs {
        ApproveAaveSupplyArgs {
            chain_id: DEFAULT_DESTINATION_CHAIN, // @dev cannot supply to AAVE in source chain
            amount: DEFAULT_AMOUNT,
            partial_transaction: build_partial_tx(),
        }
    }
}
