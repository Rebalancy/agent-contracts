use crate::encoders;
use crate::types::{
    AaveArgs, AaveConfig, ApproveAaveSupplyArgs, ApproveCctpBurnArgs, CCTPBurnArgs, CCTPMintArgs,
    RebalancerArgs,
};
use alloy_primitives::{Address, B256, U256};
use std::str::FromStr;

pub fn build_cctp_approve_burn_tx(args: ApproveCctpBurnArgs) -> Vec<u8> {
    let input = encoders::cctp::usdc::encode_approve(
        Address::from_str(&args.spender).expect("Invalid spender address"),
        U256::from(args.amount),
    );
    input
}

pub fn build_cctp_burn_tx(args: CCTPBurnArgs) -> Vec<u8> {
    let input = encoders::cctp::messenger::encode_deposit_for_burn(
        U256::from(args.amount),
        args.destination_domain,
        B256::from_str(&args.mint_recipient).expect("Invalid recipient"),
        Address::from_str(&args.burn_token).expect("Invalid token address"),
        B256::from_str(&args.destination_caller).expect("Invalid destination caller"),
        U256::from(args.max_fee),
        args.min_finality_threshold,
    );
    input
}

pub fn build_cctp_mint_tx(args: CCTPMintArgs) -> Vec<u8> {
    let input = encoders::cctp::transmitter::encode_receive_message(
        args.message.clone(),
        args.attestation.clone(),
    );
    input
}

pub fn build_aave_approve_supply_tx(args: ApproveAaveSupplyArgs) -> Vec<u8> {
    let input = encoders::cctp::usdc::encode_approve(
        Address::from_str(&args.spender).expect("Invalid spender address"),
        U256::from(args.amount),
    );
    input
}

pub fn build_aave_supply_tx(args: AaveArgs, config: AaveConfig) -> Vec<u8> {
    let input = encoders::aave::lending_pool::encode_supply(
        Address::from_str(&config.asset).expect("Invalid asset address"),
        U256::from(args.amount),
        Address::from_str(&config.on_behalf_of).expect("Invalid on_behalf_of address"),
        config.referral_code,
    );
    input
}

pub fn build_aave_withdraw_tx(args: AaveArgs, config: AaveConfig) -> Vec<u8> {
    let input = encoders::aave::lending_pool::encode_withdraw(
        Address::from_str(&config.asset).expect("Invalid asset address"),
        U256::from(args.amount),
        Address::from_str(&config.on_behalf_of).expect("Invalid on_behalf_of address"),
    );
    input
}

pub fn build_withdraw_for_crosschain_allocation_tx(args: RebalancerArgs) -> Vec<u8> {
    let input = encoders::rebalancer::vault::encode_withdraw_for_crosschain_allocation(
        args.amount,
        args.cross_chain_a_token_balance.unwrap_or(0),
    );
    input
}

pub fn build_return_funds_tx(args: RebalancerArgs) -> Vec<u8> {
    let input = encoders::rebalancer::vault::encode_return_funds(
        args.amount,
        args.cross_chain_a_token_balance.unwrap_or(0),
    );
    input
}

pub fn build_approve_vault_to_manage_agents_usdc_tx(spender: String) -> Vec<u8> {
    encoders::cctp::usdc::encode_approve(
        Address::from_str(&spender).expect("Invalid spender address"),
        U256::MAX,
    )
}

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
    fn test_build_aave_supply_tx() {
        let args = AaveArgs {
            amount: 500,
            partial_transaction: dummy_tx(),
        };
        let config = AaveConfig {
            asset: "87870Bca3F3fD6335C3F4ce8392D69350B4fA4E2".to_string(),
            on_behalf_of: "87870Bca3F3fD6335C3F4ce8392D69350B4fA4E2".to_string(),
            referral_code: 0,
            lending_pool_address: "87870Bca3F3fD6335C3F4ce8392D69350B4fA4E2".to_string(),
        };
        let payload = build_aave_supply_tx(args, config);

        assert!(!payload.is_empty());
    }

    #[test]
    fn test_build_aave_withdraw_tx() {
        let args = AaveArgs {
            amount: 500,
            partial_transaction: dummy_tx(),
        };
        let config = AaveConfig {
            asset: "87870Bca3F3fD6335C3F4ce8392D69350B4fA4E2".to_string(),
            on_behalf_of: "87870Bca3F3fD6335C3F4ce8392D69350B4fA4E2".to_string(),
            referral_code: 0,
            lending_pool_address: "87870Bca3F3fD6335C3F4ce8392D69350B4fA4E2".to_string(),
        };
        let payload = build_aave_withdraw_tx(args, config);
        assert!(!payload.is_empty());
    }

    #[test]
    fn test_build_cctp_burn_tx() {
        let args = CCTPBurnArgs {
            amount: 1000,
            destination_domain: 100,
            mint_recipient: format!("{:0>64}", "87870Bca3F3fD6335C3F4ce8392D69350B4fA4E2"),
            burn_token: "87870Bca3F3fD6335C3F4ce8392D69350B4fA4E2".to_string(),
            destination_caller: format!("{:0>64}", "87870Bca3F3fD6335C3F4ce8392D69350B4fA4E2"),
            max_fee: 0,
            min_finality_threshold: 0,
            partial_burn_transaction: dummy_tx(),
        };
        let payload = build_cctp_burn_tx(args);
        println!("burn payload: {}", encode(&payload));
        assert!(!payload.is_empty());
    }

    #[test]
    fn test_build_cctp_mint_tx() {
        let args = CCTPMintArgs {
            partial_mint_transaction: dummy_tx(),
            message: vec![0xde, 0xad],
            attestation: vec![0xbe, 0xef],
        };
        let payload = build_cctp_mint_tx(args);
        println!("mint payload: {}", encode(&payload));
        assert!(!payload.is_empty());
    }

    #[test]
    fn test_build_withdraw_for_crosschain_allocation_tx() {
        let args = RebalancerArgs {
            amount: 1234,
            partial_transaction: dummy_tx(),
            cross_chain_a_token_balance: Some(5000),
        };
        let payload = build_withdraw_for_crosschain_allocation_tx(args);
        println!("withdraw payload: {}", encode(&payload));
        assert!(!payload.is_empty());
    }

    #[test]
    fn test_build_return_funds_tx() {
        let args = RebalancerArgs {
            amount: 1234,
            partial_transaction: dummy_tx(),
            cross_chain_a_token_balance: Some(5000),
        };
        let payload = build_return_funds_tx(args);
        println!("return funds payload: {}", encode(&payload));
        assert!(!payload.is_empty());
    }
}
