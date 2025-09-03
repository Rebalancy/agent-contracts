use alloy::network::Ethereum;
use alloy::primitives::Address;
use alloy::providers::layers::AnvilProvider;
use alloy::providers::RootProvider;
use alloy::providers::{ext::AnvilApi, Provider, ProviderBuilder};
use alloy::transports::http::{Client, Http};
use near_primitives::action::FunctionCallAction;
use omni_transaction::evm::EVMTransaction;
use serde_json::json;

mod utils;

use crate::utils::account_config::{get_user_account_info_from_file, NearAccount};
use crate::utils::alchemy_provider::{AlchemyFactoryProvider, IProviderFactory, Network};
use crate::utils::friendly_json_rpc_client::near_network_config::NearNetworkConfig;
use crate::utils::friendly_json_rpc_client::FriendlyNearJsonRpcClient;
use crate::utils::mpc::addresses::{
    convert_string_to_public_key, derive_epsilon, derive_key, public_key_to_evm_address,
    ROOT_PUBLIC_KEY,
};
use dotenvy::dotenv;
use shade_agent_contract::types::{ActivityLog, ChainId, PayloadType};
use std::collections::HashMap;
use std::env;
use std::str::FromStr;

const OPTIMISM_CHAIN_ID_SEPOLIA: u64 = 11155420;
const OPTIMISM_DOMAIN: u32 = 2;
const ARBITRUM_CHAIN_ID_SEPOLIA: u64 = 421614;
const ARBITRUM_DOMAIN: u32 = 3;
const USDC_AMOUNT: u64 = 1;
const MIN_FINALITY_THRESHOLD: u64 = 1000;
const USDC_ARBITRUM_SEPOLIA: &str = "0x75faf114eafb1BDbe2F0316DF893fd58CE46AA4d"; // USDC on Arbitrum Sepolia
const USDC_OPTIMISM_SEPOLIA: &str = "0x5fd84259d66Cd46123540766Be93DFE6D43130D7"; // USDC on Optimism Sepolia
const LENDING_POOL_ARBITRUM_SEPOLIA: &str = "0xBfC91D59fdAA134A4ED45f7B584cAf96D7792Eff"; // Aave Lending Pool on Arbitrum Sepolia
const LENDING_POOL_OPTIMISM_SEPOLIA: &str = "0xb50201558B00496A145fE76f7424749556E326D8"; // Aave Lending Pool on Optimism Sepolia
const MESSENGER_ADDRESS_ARBITRUM_SEPOLIA: &str = "0x8FE6B999Dc680CcFDD5Bf7EB0974218be2542DAA"; // CCTP Messenger on Arbitrum Sepolia
const MESSENGER_ADDRESS_OPTIMISM_SEPOLIA: &str = "0x8FE6B999Dc680CcFDD5Bf7EB0974218be2542DAA"; // CCTP Messenger on Optimism Sepolia
const TRANSMITTER_ADDRESS_ARBITRUM_SEPOLIA: &str = "0xe737e5cebeeba77efe34d4aa090756590b1ce275"; // CCTP Transmitter on Arbitrum Sepolia
const TRANSMITTER_ADDRESS_OPTIMISM_SEPOLIA: &str = "0xE737e5cEBEEBa77EFE34D4aa090756590b1CE275"; // CCTP Transmitter on Optimism Sepolia
const VAULT_ADDRESS_ARBITRUM_SEPOLIA: &str = "0x858a8AFff11BfCCB61e69da87EBa1eCCCC34C640"; // Rebalancer Vault on Arbitrum Sepolia
const ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000";
const SOURCE_CHAIN_ID: u64 = ARBITRUM_CHAIN_ID_SEPOLIA;
const PATH: &str = "ethereum-1";

async fn deploy_and_initialise(
    deployer_account: NearAccount,
) -> Result<(), Box<dyn std::error::Error>> {
    let wasm_bytes = include_bytes!("../target/near/shade_agent_contract.wasm").to_vec();

    let friendly_json_rpc_client =
        FriendlyNearJsonRpcClient::new(NearNetworkConfig::Testnet, deployer_account.clone());

    let result = friendly_json_rpc_client.deploy_contract(wasm_bytes).await?;
    println!("Deploy result: {:?}", result);
    println!("Contract deployed at: {}\n", deployer_account.account_id);

    let init_args = json!({
        "source_chain": SOURCE_CHAIN_ID,
        "configs": [{
            "chain_id": SOURCE_CHAIN_ID,
            "config": {
                "aave": {
                    "asset": USDC_ARBITRUM_SEPOLIA, // https://developers.circle.com/stablecoins/usdc-contract-addresses#testnet
                    "on_behalf_of": ZERO_ADDRESS,
                    "referral_code": 0,
                    "lending_pool_address": LENDING_POOL_ARBITRUM_SEPOLIA, // Aave Lending Pool on Arbitrum Sepolia
                },
                "cctp": {
                    "messenger_address": MESSENGER_ADDRESS_ARBITRUM_SEPOLIA, // CCTP Messenger on Arbitrum Sepolia
                    "transmitter_address": TRANSMITTER_ADDRESS_ARBITRUM_SEPOLIA, // CCTP Transmitter on Arbitrum Sepolia
                },
                "rebalancer": {
                    "vault_address": VAULT_ADDRESS_ARBITRUM_SEPOLIA
                }
            }
        },
        {
            "chain_id": OPTIMISM_CHAIN_ID_SEPOLIA, // Optimism Sepolia
            "config": {
                "aave": {
                    "asset": USDC_OPTIMISM_SEPOLIA, // https://developers.circle.com/stablecoins/usdc-contract-addresses#testnet
                    "on_behalf_of": ZERO_ADDRESS,
                    "referral_code": 0,
                    "lending_pool_address":LENDING_POOL_OPTIMISM_SEPOLIA, // Aave Lending Pool on Optimism Sepolia
                },
                "cctp": {
                    "messenger_address": MESSENGER_ADDRESS_OPTIMISM_SEPOLIA, // CCTP Messenger on Optimism Sepolia
                    "transmitter_address": TRANSMITTER_ADDRESS_OPTIMISM_SEPOLIA, // CCTP Transmitter on Optimism Sepolia
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

    Ok(())
}

fn get_agent_address(deployer_account: NearAccount) -> Address {
    let epsilon = derive_epsilon(&deployer_account.account_id, PATH);
    let public_key = convert_string_to_public_key(ROOT_PUBLIC_KEY).unwrap();
    let derived_public_key = derive_key(public_key, epsilon);
    let agent_address = public_key_to_evm_address(derived_public_key);

    Address::from_str(&agent_address).unwrap()
}

async fn execute_rebalance_steps_on_destionation_chain(
    deployer_account: NearAccount,
    rpc_url: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let provider = ProviderBuilder::new().on_anvil_with_config(|anvil| anvil.fork(rpc_url.clone()));
    let agent_address = get_agent_address(deployer_account.clone());

    let friendly_json_rpc_client =
        FriendlyNearJsonRpcClient::new(NearNetworkConfig::Testnet, deployer_account.clone());

    // @dev: Ideally we should try to get the attestation from the Circle API here but we just test the signature creation.

    // 4) Mint on destination chain
    let mint_for_bridge_args = json!({
        "args": {
            "message": [], // TODO: Fill in with actual message from Circle API
            "attestation": [], // TODO: Fill in with actual attestation from Circle API
            "partial_mint_transaction": build_transaction(&provider, agent_address).await?,
        },
        "callback_gas_tgas": 10,
    });

    let mint_on_destination_chain_result = friendly_json_rpc_client
        .send_action(FunctionCallAction {
            method_name: "build_cctp_mint_tx".to_string(),
            args: mint_for_bridge_args.to_string().into_bytes(), // Convert directly to Vec<u8>
            gas: 300000000000000,
            deposit: 0,
        })
        .await?;

    println!(
        "Mint on destination chain result: {:?}",
        mint_on_destination_chain_result
    );

    // 5) Deposit to Aave on destination chain
    let deposit_to_aave_args = json!({
        "args": {
            "amount": USDC_AMOUNT,
            "partial_transaction": build_transaction(&provider, agent_address).await?
        },
        "callback_gas_tgas": 10
    });
    let deposit_to_aave_result = friendly_json_rpc_client
        .send_action(FunctionCallAction {
            method_name: "build_aave_supply_tx".to_string(),
            args: deposit_to_aave_args.to_string().into_bytes(), // Convert directly to Vec<u8>
            gas: 300000000000000,
            deposit: 0,
        })
        .await?;

    println!(
        "Deposit to Aave on destination chain result: {:?}",
        deposit_to_aave_result
    );

    // 6) TODO: Complete rebalance???

    Ok(())
}

async fn burn_for_bridge(
    deployer_account: NearAccount,
    rpc_url: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let provider = ProviderBuilder::new().on_anvil_with_config(|anvil| anvil.fork(rpc_url.clone()));
    let agent_address = get_agent_address(deployer_account.clone());

    let friendly_json_rpc_client =
        FriendlyNearJsonRpcClient::new(NearNetworkConfig::Testnet, deployer_account.clone());

    let partial_burn_tx = build_transaction(&provider, agent_address).await?;

    let burn_for_bridge_args = json!({
        "args": {
            "amount": USDC_AMOUNT,
            "destination_domain": OPTIMISM_DOMAIN,
            "mint_recipient": address_to_bytes32_string(&agent_address.to_string()),
            "burn_token": USDC_ARBITRUM_SEPOLIA,
            "destination_caller": address_to_bytes32_string(&agent_address.to_string()),
            "max_fee": 0.99 * 1_000_000.0, // 0.99 USDC in 6 decimal places
            "min_finality_threshold": MIN_FINALITY_THRESHOLD,
            "partial_burn_transaction": partial_burn_tx
        },
        "callback_gas_tgas": 10,
    });

    let burn_for_bridge_result = friendly_json_rpc_client
        .send_action(FunctionCallAction {
            method_name: "build_cctp_burn_tx".to_string(),
            args: burn_for_bridge_args.to_string().into_bytes(), // Convert directly to Vec<u8>
            gas: 300000000000000,
            deposit: 0,
        })
        .await?;
    println!("Burn for bridge result: {:?}", burn_for_bridge_result);

    Ok(())
}

async fn start_rebalance(deployer_account: NearAccount) -> Result<(), Box<dyn std::error::Error>> {
    let friendly_json_rpc_client =
        FriendlyNearJsonRpcClient::new(NearNetworkConfig::Testnet, deployer_account.clone());

    // 1) Start rebalance
    let start_rebalancer_args = json!({
        "flow": "RebalancerToAave",
        "source_chain": ARBITRUM_CHAIN_ID_SEPOLIA,
        "destination_chain": OPTIMISM_CHAIN_ID_SEPOLIA,
        "expected_amount": USDC_AMOUNT,
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

    Ok(())
}

async fn withdraw_funds_for_allocation(
    deployer_account: NearAccount,
    rpc_url: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let provider = ProviderBuilder::new().on_anvil_with_config(|anvil| anvil.fork(rpc_url.clone()));
    let agent_address = get_agent_address(deployer_account.clone());

    let friendly_json_rpc_client =
        FriendlyNearJsonRpcClient::new(NearNetworkConfig::Testnet, deployer_account.clone());

    let tx_withdraw_for_allocation = build_transaction(&provider, agent_address).await?;
    print!(
        "Built tx_withdraw_for_allocation: {:?}",
        tx_withdraw_for_allocation
    );
    let withdraw_for_allocation_args = json!({
        "rebalancer_args": {
            "amount": USDC_AMOUNT,
            "partial_transaction": tx_withdraw_for_allocation,
            "cross_chain_a_token_balance": 1
        },
        "callback_gas_tgas": 10
    });

    let withdraw_for_allocation_result = friendly_json_rpc_client
        .send_action(FunctionCallAction {
            method_name: "build_withdraw_for_crosschain_allocation_tx".to_string(),
            args: withdraw_for_allocation_args.to_string().into_bytes(), // Convert directly to Vec<u8>
            gas: 300000000000000,
            deposit: 0,
        })
        .await?;

    println!(
        "Withdraw for allocation result: {:?}",
        withdraw_for_allocation_result
    );

    Ok(())
}

async fn build_transaction(
    provider: &AnvilProvider<RootProvider<Http<Client>, Ethereum>, Http<Client>>,
    agent_address: Address,
) -> Result<EVMTransaction, Box<dyn std::error::Error>> {
    let info = provider.anvil_node_info().await?;
    let chain_id: u64 = info.environment.chain_id;
    let nonce: u64 = provider.get_account(agent_address).await?.nonce;
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
    Ok(empty_tx)
}

fn address_to_bytes32_string(addr: &str) -> String {
    let addr = addr.strip_prefix("0x").unwrap_or(addr);
    assert_eq!(addr.len(), 40, "Address must be 20 bytes (40 hex chars)");

    let padded = format!("{:0>64}", addr); // pad left with zeros to reach 64 chars
    println!("Padded address_to_bytes32_string: {}", padded);
    padded
}

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

async fn test_get_signed_transactions() -> Result<(), Box<dyn std::error::Error>> {
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
    let provider = ProviderBuilder::new()
        .on_anvil_with_config(|anvil| anvil.fork(rpc_url).chain_id(ARBITRUM_CHAIN_ID_SEPOLIA));

    // Get node info using the Anvil API.
    let info = provider.anvil_node_info().await?;

    println!("Node info: {info:#?}");

    assert_eq!(info.environment.chain_id, ARBITRUM_CHAIN_ID_SEPOLIA);
    assert_eq!(info.fork_config.fork_url, Some(rpc_url.to_string()));

    let rebalancer_tx_payload = grouped
        .get(&PayloadType::RebalancerWithdrawToAllocate)
        .expect("RebalancerWithdrawToAllocate payload not found");

    match provider.send_raw_transaction(rebalancer_tx_payload).await {
        Ok(tx_hash) => {
            println!("Transaction sent successfully. Hash: {:?}", tx_hash);
        }
        Err(err) => {
            eprintln!("Failed to send transaction: {err}");
        }
    }
    Ok(())
}

async fn test_get_allocations() -> Result<(), Box<dyn std::error::Error>> {
    let deployer_account = get_user_account_info_from_file(None)?;

    let friendly_json_rpc_client =
        FriendlyNearJsonRpcClient::new(NearNetworkConfig::Testnet, deployer_account.clone());

    let allocations = friendly_json_rpc_client
        .call_contract::<Vec<(ChainId, u128)>>("get_allocations", json!({}))
        .await?;

    println!("Allocations: {:?}", allocations);

    Ok(())
}

#[tokio::test]
async fn full_flow_test() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let alchemy_api_key = env::var("ALCHEMY_API_KEY").expect("ALCHEMY_API_KEY must be set");
    let alchemy_provider = AlchemyFactoryProvider::new(alchemy_api_key);
    let alchemy_url = alchemy_provider.alchemy_url_for(Network::ArbitrumSepolia)?;
    assert!(alchemy_provider.is_network_supported(&Network::ArbitrumSepolia));

    let deployer_account: NearAccount = get_user_account_info_from_file(None)?;

    // TODO: Split into individual actions

    // TODO: make a cache thing to avoid redeploying if it was deployed

    deploy_and_initialise(deployer_account.clone()).await?;
    start_rebalance(deployer_account.clone()).await?;
    withdraw_funds_for_allocation(deployer_account.clone(), alchemy_url.clone()).await?;
    // burn_for_bridge(deployer_account.clone(), alchemy_url.clone()).await?;
    // execute_rebalance_steps_on_sourche_chain(deployer_account.clone()).await?;
    // execute_rebalance_steps_on_destionation_chain(deployer_account.clone()).await?;
    // test_get_activity().await?;
    // test_get_allocations().await?;
    // test_get_signed_transactions().await?;
    println!("Full flow test completed successfully.");
    Ok(())
}
