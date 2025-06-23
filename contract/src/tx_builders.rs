// In `src/tx_builders.rs`
use crate::encoders;
use crate::types::{AaveArgs, AaveConfig, CCTPArgs, RebalancerArgs};
use alloy_primitives::{Address, B256, U256};
use std::str::FromStr;

pub fn build_invest_tx(args: RebalancerArgs) -> Vec<u8> {
    let input = encoders::rebalancer::vault::encode_invest(args.amount);
    let mut tx = args.partial_transaction;
    tx.input = input;
    tx.build_for_signing().try_into().unwrap()
}

pub fn build_cctp_burn_tx(args: CCTPArgs) -> Vec<u8> {
    let input = encoders::cctp::messenger::encode_deposit_for_burn(
        U256::from(args.amount),
        args.destination_domain,
        B256::from_str(&args.mint_recipient).expect("Invalid recipient"),
        Address::from_str(&args.burn_token).expect("Invalid token address"),
        B256::from_str(&args.destination_caller).expect("Invalid destination caller"),
        U256::from(args.max_fee),
        args.min_finality_threshold,
    );
    let mut tx = args.partial_burn_transaction;
    tx.input = input;
    tx.build_for_signing().try_into().unwrap()
}

pub fn build_cctp_mint_tx(args: CCTPArgs) -> Vec<u8> {
    let input = encoders::cctp::transmitter::encode_receive_message(
        args.message.clone(),
        args.attestation.clone(),
    );
    let mut tx = args.partial_mint_transaction;
    tx.input = input;
    tx.build_for_signing().try_into().unwrap()
}

pub fn build_aave_tx(args: AaveArgs, config: AaveConfig) -> Vec<u8> {
    let input = encoders::aave::lending_pool::encode_supply(
        Address::from_str(&config.asset).expect("Invalid asset address"),
        U256::from(args.amount),
        Address::from_str(&config.on_behalf_of).expect("Invalid on_behalf_of address"),
        config.referral_code,
    );
    let mut tx = args.partial_transaction;
    tx.input = input;
    tx.build_for_signing().try_into().unwrap()
}

// In `tests/test_tx_builders.rs`
#[cfg(test)]
mod tests {
    use super::*;
    use hex::encode;
    use omni_transaction::evm::{utils::parse_eth_address, EVMTransaction};

    fn dummy_tx() -> EVMTransaction {
        let to_address = parse_eth_address("87870Bca3F3fD6335C3F4ce8392D69350B4fA4E2"); // Aave Lending Pool

        EVMTransaction {
            chain_id: 1,
            nonce: 0,
            gas_limit: 100000,
            max_fee_per_gas: 1000000000,
            max_priority_fee_per_gas: 100000000,
            to: Some(to_address),
            value: 0,
            input: vec![],
            access_list: vec![],
        }
    }

    #[test]
    fn test_build_invest_tx() {
        let args = RebalancerArgs {
            amount: 1234,
            source_chain: 1,
            destination_chain: 1,
            partial_transaction: dummy_tx(),
        };
        let payload = build_invest_tx(args);
        println!("invest payload: {}", encode(&payload));
        assert!(!payload.is_empty());
    }

    #[test]
    fn test_build_cctp_burn_tx() {
        let args = CCTPArgs {
            amount: 1000,
            destination_domain: 100,
            mint_recipient: format!("{:0>64}", "87870Bca3F3fD6335C3F4ce8392D69350B4fA4E2"),
            burn_token: "87870Bca3F3fD6335C3F4ce8392D69350B4fA4E2".to_string(),
            destination_caller: format!("{:0>64}", "87870Bca3F3fD6335C3F4ce8392D69350B4fA4E2"),
            max_fee: 0,
            min_finality_threshold: 0,
            message: vec![],
            attestation: vec![],
            partial_burn_transaction: dummy_tx(),
            partial_mint_transaction: dummy_tx(),
        };
        let payload = build_cctp_burn_tx(args);
        println!("burn payload: {}", encode(&payload));
        assert!(!payload.is_empty());
    }

    // #[test]
    // fn test_build_cctp_mint_tx() {
    //     let args = CCTPArgs {
    //         message: vec![0xde, 0xad],
    //         attestation: vec![0xbe, 0xef],
    //         ..Default::default()
    //     };
    //     let payload = build_cctp_mint_tx(&args);
    //     println!("mint payload: {}", encode(&payload));
    //     assert!(!payload.is_empty());
    // }

    #[test]
    fn test_build_aave_tx() {
        let args = AaveArgs {
            amount: 500,
            partial_transaction: dummy_tx(),
        };
        let config = AaveConfig {
            asset: "87870Bca3F3fD6335C3F4ce8392D69350B4fA4E2".to_string(),
            on_behalf_of: "87870Bca3F3fD6335C3F4ce8392D69350B4fA4E2".to_string(),
            referral_code: 0,
        };
        let payload = build_aave_tx(args, config);
        println!("aave payload: {}", encode(&payload));
        assert!(!payload.is_empty());
    }
}
