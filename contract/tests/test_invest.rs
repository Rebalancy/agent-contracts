use alloy::providers::{ext::AnvilApi, Provider, ProviderBuilder};
use near_primitives::action::FunctionCallAction;
use omni_transaction::evm::EVMTransaction;
use serde_json::json;

mod utils;

use crate::utils::account_config::get_user_account_info_from_file;
use crate::utils::conversion::to_usdc_units;
use crate::utils::friendly_json_rpc_client::near_network_config::NearNetworkConfig;
use crate::utils::friendly_json_rpc_client::FriendlyNearJsonRpcClient;

use shade_agent_contract::types::{ActivityLog, ChainId, PayloadType};
use std::collections::HashMap;

const BASE_CHAIN_ID_SEPOLIA: u64 = 84532;
const ETHEREUM_CHAIN_ID_SEPOLIA: u64 = 111155111;
const BASE_DOMAIN: u32 = 6;
const ETHEREUM_DOMAIN: u32 = 0;
const USDC_AMOUNT: u64 = 1;
const MIN_FINALITY_THRESHOLD: u64 = 1000;
const AGENT_ADDRESS: &str = "0xD5aC5A88dd3F1FE5dcC3ac97B512Faeb48d06AF0";
const USDC_BASE_SEPOLIA: &str = "0x036CbD53842c5426634e7929541eC2318f3dCF7e"; // USDC on Base Sepolia
const USDC_ETHEREUM_SEPOLIA: &str = "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238"; // USDC on Ethereum Sepolia
const LENDING_POOL_BASE_SEPOLIA: &str = "0x8bAB6d1b75f19e9eD9fCe8b9BD338844fF79aE27"; // Aave Lending Pool on Base Sepolia
const LENDING_POOL_ETHEREUM_SEPOLIA: &str = "0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951"; // Aave Lending Pool on Ethereum Sepolia
const MESSENGER_ADDRESS_BASE_SEPOLIA: &str = "0x8fe6b999dc680ccfdd5bf7eb0974218be2542daa"; // CCTP Messenger on Base Sepolia
const MESSENGER_ADDRESS_ETHEREUM_SEPOLIA: &str = "0x8fe6b999dc680ccfdd5bf7eb0974218be2542daa"; // CCTP Messenger on Ethereum Sepolia
const TRANSMITTER_ADDRESS_BASE_SEPOLIA: &str = "0xe737e5cebeeba77efe34d4aa090756590b1ce275"; // CCTP Transmitter on Base Sepolia
const TRANSMITTER_ADDRESS_ETHEREUM_SEPOLIA: &str = "0xe737e5cebeeba77efe34d4aa090756590b1ce275"; // CCTP Transmitter on Ethereum Sepolia
const VAULT_ADDRESS_BASE_SEPOLIA: &str = "0x565FDe3703d1bCc7Cbe161488ee1498ae429A145"; // Rebalancer Vault on Base Sepolia
const ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000";

#[tokio::test]
#[ignore]
async fn test_invest() -> Result<(), Box<dyn std::error::Error>> {
    let deployer_account = get_user_account_info_from_file(None)?;

    let wasm_bytes = include_bytes!("../target/near/shade_agent_contract.wasm").to_vec();

    let friendly_json_rpc_client =
        FriendlyNearJsonRpcClient::new(NearNetworkConfig::Testnet, deployer_account.clone());

    let result = friendly_json_rpc_client.deploy_contract(wasm_bytes).await?;
    println!("Deploy result: {:?}", result);
    println!("Contract deployed at: {}\n", deployer_account.account_id);

    let init_args = json!({
        "source_chain": BASE_CHAIN_ID_SEPOLIA, // Base Sepolia
        "configs": [{
            "chain_id": BASE_CHAIN_ID_SEPOLIA,
            "config": {
                "aave": {
                    "asset": USDC_BASE_SEPOLIA, // https://developers.circle.com/stablecoins/usdc-contract-addresses#testnet
                    "on_behalf_of": ZERO_ADDRESS,
                    "referral_code": 0,
                    "lending_pool_address": LENDING_POOL_BASE_SEPOLIA, // Aave Lending Pool on Base Sepolia
                },
                "cctp": {
                    "messenger_address": MESSENGER_ADDRESS_BASE_SEPOLIA, // CCTP Messenger on Base Sepolia
                    "transmitter_address": TRANSMITTER_ADDRESS_BASE_SEPOLIA, // CCTP Transmitter on Base Sepolia
                },
                "rebalancer": {
                    "vault_address": VAULT_ADDRESS_BASE_SEPOLIA
                }
            }
        },
        {
            "chain_id": ETHEREUM_CHAIN_ID_SEPOLIA, // Ethereum Sepolia
            "config": {
                "aave": {
                    "asset": USDC_ETHEREUM_SEPOLIA, // https://developers.circle.com/stablecoins/usdc-contract-addresses#testnet
                    "on_behalf_of": ZERO_ADDRESS,
                    "referral_code": 0,
                    "lending_pool_address":LENDING_POOL_ETHEREUM_SEPOLIA, // Aave Lending Pool on Ethereum Sepolia
                },
                "cctp": {
                    "messenger_address": MESSENGER_ADDRESS_ETHEREUM_SEPOLIA, // CCTP Messenger on Ethereum Sepolia
                    "transmitter_address": TRANSMITTER_ADDRESS_ETHEREUM_SEPOLIA, // CCTP Transmitter on Ethereum Sepolia
                },
                "rebalancer": {
                    "vault_address": ZERO_ADDRESS
                }
            }
        }]
    });

    let init_result = friendly_json_rpc_client
        .send_action(FunctionCallAction {
            method_name: "init".to_string(),
            args: init_args.to_string().into_bytes(), // Convert directly to Vec<u8>
            gas: 300000000000000,
            deposit: 0,
        })
        .await?;

    println!("Init result: {:?}", init_result);

    // TODO: Use this to get nonces

    // Spin up a forked Anvil node. (Ensure `anvil` is available in $PATH)
    let rpc_url = "https://base-sepolia.g.alchemy.com/v2/JmXSjhNebn2v_jTEgLoqKyf4q_H8EwIn";
    let provider = ProviderBuilder::new().on_anvil_with_config(|anvil| anvil.fork(rpc_url));

    // Get node info using the Anvil API.
    let info = provider.anvil_node_info().await?;

    println!("Node info: {info:#?}");

    assert_eq!(info.environment.chain_id, BASE_CHAIN_ID_SEPOLIA);
    assert_eq!(info.fork_config.fork_url, Some(rpc_url.to_string()));

    // Prepare Ethereum transaction
    let chain_id: u64 = 1;
    let nonce: u64 = 0x42; // TODO: this has to be dynamic....
    let gas_limit = 44386;
    let max_fee_per_gas = 0x4a817c800;
    let max_priority_fee_per_gas = 0x3b9aca00;
    let value = 0;

    let empty_tx = EVMTransaction {
        chain_id,
        nonce,
        gas_limit,
        max_fee_per_gas,
        max_priority_fee_per_gas,
        to: None,
        value,
        input: vec![],
        access_list: vec![],
    };

    let start_rebalancer_args = json!({
        "source_chain": BASE_CHAIN_ID_SEPOLIA,
        "destination_chain": ETHEREUM_CHAIN_ID_SEPOLIA,
        "rebalancer_args": {
            "amount": USDC_AMOUNT,
            "source_chain": BASE_CHAIN_ID_SEPOLIA,
            "destination_chain": ETHEREUM_CHAIN_ID_SEPOLIA,
            "partial_transaction": empty_tx
        },
        "cctp_args": {
            "amount": USDC_AMOUNT,
            "destination_domain": ETHEREUM_DOMAIN,
            "mint_recipient": address_to_bytes32_string(AGENT_ADDRESS),
            "burn_token": USDC_BASE_SEPOLIA,
            "destination_caller": address_to_bytes32_string(AGENT_ADDRESS),
            "max_fee": to_usdc_units(0.99),
            "min_finality_threshold": MIN_FINALITY_THRESHOLD,
            "message": [],
            "attestation": [],
            "partial_burn_transaction": empty_tx,
            "partial_mint_transaction": empty_tx
        },
        "gas_for_rebalancer": 10,
        "gas_for_cctp_burn": 10,
    });

    let start_rebalance_result = friendly_json_rpc_client
        .send_action(FunctionCallAction {
            method_name: "start_rebalance".to_string(),
            args: start_rebalancer_args.to_string().into_bytes(), // Convert directly to Vec<u8>
            gas: 300000000000000,
            deposit: 0,
        })
        .await?;

    println!("Start rebalance result: {:?}", start_rebalance_result);

    let complete_rebalancer_args = json!({
        "cctp_args": {
            "amount": USDC_AMOUNT,
            "destination_domain": ETHEREUM_DOMAIN,
            "mint_recipient": address_to_bytes32_string(AGENT_ADDRESS),
            "burn_token": USDC_BASE_SEPOLIA,
            "destination_caller": address_to_bytes32_string(AGENT_ADDRESS),
            "max_fee": to_usdc_units(0.99),
            "min_finality_threshold": MIN_FINALITY_THRESHOLD,
            "message": [],
            "attestation": [],
            "partial_burn_transaction": empty_tx,
            "partial_mint_transaction": empty_tx
        },
        "aave_args": {
            "amount": USDC_AMOUNT,
            "partial_transaction": empty_tx
        },
        "gas_cctp_mint": 10,
        "gas_aave": 10,
    });

    let complete_rebalancer_result = friendly_json_rpc_client
        .send_action(FunctionCallAction {
            method_name: "complete_rebalance".to_string(),
            args: complete_rebalancer_args.to_string().into_bytes(), // Convert directly to Vec<u8>
            gas: 300000000000000,
            deposit: 0,
        })
        .await?;

    println!(
        "Complete rebalance result: {:?}",
        complete_rebalancer_result
    );

    Ok(())
}

fn address_to_bytes32_string(addr: &str) -> String {
    let addr = addr.strip_prefix("0x").unwrap_or(addr);
    assert_eq!(addr.len(), 40, "Address must be 20 bytes (40 hex chars)");

    let padded = format!("{:0>64}", addr); // pad left with zeros to reach 64 chars
    println!("Padded address_to_bytes32_string: {}", padded);
    padded
}

#[tokio::test]
async fn test_get_activity() -> Result<(), Box<dyn std::error::Error>> {
    let deployer_account = get_user_account_info_from_file(None)?;

    let friendly_json_rpc_client =
        FriendlyNearJsonRpcClient::new(NearNetworkConfig::Testnet, deployer_account.clone());

    let result = friendly_json_rpc_client
        .call_contract::<Vec<ActivityLog>>(
            "get_latest_logs",
            json!({
                "count": 10
            }),
        )
        .await?;

    println!("Get latest logs result: {:?}", result);

    Ok(())
}

#[tokio::test]
async fn get_signed_transactions() -> Result<(), Box<dyn std::error::Error>> {
    let deployer_account = get_user_account_info_from_file(None)?;

    let friendly_json_rpc_client =
        FriendlyNearJsonRpcClient::new(NearNetworkConfig::Testnet, deployer_account.clone());

    let payloads = friendly_json_rpc_client
        .call_contract::<Vec<Vec<u8>>>(
            "get_signed_transactions",
            json!({
            "nonce": 0
            }),
        )
        .await?;

    println!("Get signed transactions result: {:?}", payloads);

    let mut grouped: HashMap<PayloadType, Vec<u8>> = HashMap::new();

    for payload in payloads {
        if payload.is_empty() {
            continue;
        }
        let payload_type = PayloadType::from(payload[0]);
        let raw_tx = payload[1..].to_vec(); // tx without the first byte (the type)

        // only insert if the type is not already present
        // this ensures that we do not have duplicate transaction types
        let already = grouped.insert(payload_type, raw_tx);
        assert!(already.is_none(), "Duplicate transaction type found!");
    }

    for (ptype, tx) in grouped.clone() {
        println!("Tx type {:?}: 0x{}", ptype, hex::encode(tx));
    }
    // Spin up a forked Anvil node. (Ensure `anvil` is available in $PATH)
    let rpc_url = "https://base-sepolia.g.alchemy.com/v2/JmXSjhNebn2v_jTEgLoqKyf4q_H8EwIn";
    let provider = ProviderBuilder::new().on_anvil_with_config(|anvil| anvil.fork(rpc_url));

    // Get node info using the Anvil API.
    let info = provider.anvil_node_info().await?;

    println!("Node info: {info:#?}");

    assert_eq!(info.environment.chain_id, BASE_CHAIN_ID_SEPOLIA);
    assert_eq!(info.fork_config.fork_url, Some(rpc_url.to_string()));

    let rebalancer_tx_payload = grouped
        .get(&PayloadType::RebalancerInvest)
        .expect("RebalancerInvest payload not found");

    // TODO: Uncomment this to test
    // match provider.send_raw_transaction(rebalancer_tx_payload).await {
    //     Ok(tx_hash) => {
    //         println!("Transaction sent successfully. Hash: {:?}", tx_hash);
    //     }
    //     Err(err) => {
    //         eprintln!("Failed to send transaction: {err}");
    //     }
    // }
    Ok(())
}

#[tokio::test]
async fn get_allocations() -> Result<(), Box<dyn std::error::Error>> {
    let deployer_account = get_user_account_info_from_file(None)?;

    let friendly_json_rpc_client =
        FriendlyNearJsonRpcClient::new(NearNetworkConfig::Testnet, deployer_account.clone());

    let allocations = friendly_json_rpc_client
        .call_contract::<Vec<(ChainId, u128)>>("get_allocations", json!({}))
        .await?;

    println!("Allocations: {:?}", allocations);

    Ok(())
}
