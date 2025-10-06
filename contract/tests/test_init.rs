use alloy::primitives::Address;
use near_primitives::action::FunctionCallAction;
use serde_json::json;

mod utils;

use crate::utils::account_config::{get_user_account_info_from_file, NearAccount};
use crate::utils::friendly_json_rpc_client::near_network_config::NearNetworkConfig;
use crate::utils::friendly_json_rpc_client::FriendlyNearJsonRpcClient;
use crate::utils::mpc::addresses::{
    convert_string_to_public_key, derive_epsilon, derive_key, public_key_to_evm_address,
    ROOT_PUBLIC_KEY,
};
use dotenvy::dotenv;
use std::str::FromStr;

const OPTIMISM_CHAIN_ID_SEPOLIA: u64 = 11155420;
const ARBITRUM_CHAIN_ID_SEPOLIA: u64 = 421614;
const USDC_ARBITRUM_SEPOLIA: &str = "0x75faf114eafb1BDbe2F0316DF893fd58CE46AA4d"; // USDC on Arbitrum Sepolia
const USDC_OPTIMISM_SEPOLIA: &str = "0x5fd84259d66Cd46123540766Be93DFE6D43130D7"; // USDC on Optimism Sepolia
const LENDING_POOL_ARBITRUM_SEPOLIA: &str = "0xBfC91D59fdAA134A4ED45f7B584cAf96D7792Eff"; // Aave Lending Pool on Arbitrum Sepolia
const LENDING_POOL_OPTIMISM_SEPOLIA: &str = "0xb50201558B00496A145fE76f7424749556E326D8"; // Aave Lending Pool on Optimism Sepolia
const MESSENGER_ADDRESS_ARBITRUM_SEPOLIA: &str = "0x8FE6B999Dc680CcFDD5Bf7EB0974218be2542DAA"; // CCTP Messenger on Arbitrum Sepolia
const MESSENGER_ADDRESS_OPTIMISM_SEPOLIA: &str = "0x8FE6B999Dc680CcFDD5Bf7EB0974218be2542DAA"; // CCTP Messenger on Optimism Sepolia
const TRANSMITTER_ADDRESS_ARBITRUM_SEPOLIA: &str = "0xe737e5cebeeba77efe34d4aa090756590b1ce275"; // CCTP Transmitter on Arbitrum Sepolia
const TRANSMITTER_ADDRESS_OPTIMISM_SEPOLIA: &str = "0xE737e5cEBEEBa77EFE34D4aa090756590b1CE275"; // CCTP Transmitter on Optimism Sepolia
const VAULT_ADDRESS_ARBITRUM_SEPOLIA: &str = "0xcEc84a8e4000Dc7d2B64bbbe9fD3559A725B9945"; // Rebalancer Vault on Arbitrum Sepolia
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

#[tokio::test]
async fn full_flow_test() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let deployer_account: NearAccount = get_user_account_info_from_file(None)?;

    let agent_address = get_agent_address(deployer_account.clone());

    println!("Derived agent address: {:?}", agent_address);

    deploy_and_initialise(deployer_account.clone()).await?;

    println!("Initialize flow tested successfully.");

    Ok(())
}
