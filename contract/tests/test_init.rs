use alloy::primitives::Address;
use near_primitives::action::FunctionCallAction;
use serde::Deserialize;
use serde_json::{json, Value};

mod utils;

use crate::utils::account_config::{get_user_account_info_from_file, NearAccount};
use crate::utils::friendly_json_rpc_client::near_network_config::NearNetworkConfig;
use crate::utils::friendly_json_rpc_client::FriendlyNearJsonRpcClient;
use crate::utils::mpc::addresses::{
    convert_string_to_public_key, derive_epsilon, derive_key, public_key_to_evm_address,
    ROOT_PUBLIC_KEY,
};
use dotenvy::dotenv;
use std::fs;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Config {
    chain_ids: serde_json::Map<String, Value>,
    source_chain: u64,
    path: String,
    cctp_contracts: serde_json::Map<String, Value>,
    aave_contracts: serde_json::Map<String, Value>,
}

async fn deploy_and_initialise(
    deployer_account: NearAccount,
    config: Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let wasm_bytes = include_bytes!("../target/near/shade_agent_contract.wasm").to_vec();

    let zero_address = "0x0000000000000000000000000000000000000000";

    let configs_json: Vec<Value> = config
        .chain_ids
        .iter()
        .map(|(chain_id_str, _network_name)| {
            let chain_id: u64 = chain_id_str.parse().unwrap();

            let cctp = config.cctp_contracts.get(chain_id_str).and_then(|v| v.as_object()).unwrap();
            let aave = config.aave_contracts.get(chain_id_str).and_then(|v| v.as_object()).unwrap();

            json!({
                "chain_id": chain_id,
                "config": {
                    "aave": {
                        "asset": cctp.get("usdc").map(|v| v.as_str().unwrap()).unwrap_or(zero_address),
                        "on_behalf_of": zero_address,
                        "referral_code": 0,
                        "lending_pool_address": aave.get("lendingPool").map(|v| v.as_str().unwrap()).unwrap_or(zero_address)
                    },
                    "cctp": {
                        "messenger_address": cctp.get("messenger").map(|v| v.as_str().unwrap()).unwrap_or(zero_address),
                        "transmitter_address": cctp.get("transmitter").map(|v| v.as_str().unwrap()).unwrap_or(zero_address),
                        "usdc_address": cctp.get("usdc").map(|v| v.as_str().unwrap()).unwrap_or(zero_address)
                    },
                    "rebalancer": {
                        "vault_address": zero_address
                    }
                }
            })
        })
        .collect();

    let init_args = json!({
        "source_chain": config.source_chain,
        "configs": configs_json
    });

    let friendly_json_rpc_client =
        FriendlyNearJsonRpcClient::new(NearNetworkConfig::Testnet, deployer_account.clone());

    let result = friendly_json_rpc_client.deploy_contract(wasm_bytes).await?;
    println!("Deploy result: {:?}", result);
    println!("Contract deployed at: {}\n", deployer_account.account_id);

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

fn get_agent_address(deployer_account: NearAccount, path: &str) -> Address {
    let epsilon = derive_epsilon(&deployer_account.account_id, path);
    let public_key = convert_string_to_public_key(ROOT_PUBLIC_KEY).unwrap();
    let derived_public_key = derive_key(public_key, epsilon);
    let agent_address = public_key_to_evm_address(derived_public_key);

    Address::from_str(&agent_address).unwrap()
}

fn get_configuration() -> Result<Config, Box<dyn std::error::Error>> {
    let config_str = fs::read_to_string("config.json")?;
    let config: Config = serde_json::from_str(&config_str)?;

    print!("Loaded configuration: {:?}", config);
    Ok(config)
}

#[tokio::test]
async fn init() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let deployer_account: NearAccount = get_user_account_info_from_file(None)?;

    let configuration = get_configuration()?;
    let agent_address = get_agent_address(deployer_account.clone(), configuration.path.as_str());

    println!("Derived agent address: {:?}", agent_address);

    deploy_and_initialise(deployer_account.clone(), configuration).await?;

    println!("Initialize flow tested successfully.");

    Ok(())
}
