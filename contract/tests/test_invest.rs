use near_primitives::action::FunctionCallAction;
use omni_transaction::evm::{utils::parse_eth_address, EVMTransaction};
use serde_json::json;

mod utils;

use crate::utils::account_config::get_user_account_info_from_file;
use crate::utils::friendly_json_rpc_client::near_network_config::NearNetworkConfig;
use crate::utils::friendly_json_rpc_client::FriendlyNearJsonRpcClient;
use shade_agent_contract::types::{ActivityLog, PayloadType};
use std::collections::HashMap;

#[tokio::test]
async fn test_invest() -> Result<(), Box<dyn std::error::Error>> {
    let deployer_account = get_user_account_info_from_file(None)?;

    let wasm_bytes = include_bytes!("../target/near/shade_agent_contract.wasm").to_vec();

    let friendly_json_rpc_client =
        FriendlyNearJsonRpcClient::new(NearNetworkConfig::Testnet, deployer_account.clone());

    let result = friendly_json_rpc_client.deploy_contract(wasm_bytes).await?;
    println!("Deploy result: {:?}", result);
    println!("Contract deployed at: {}\n", deployer_account.account_id);

    let init_args = json!({
        "source_chain": 1u64,
        "configs": [{
            "chain_id": 1u64,
            "config": {
                "aave": {
                    "asset": "0x0000000000000000000000000000000000000001",
                    "on_behalf_of": "0x0000000000000000000000000000000000000001",
                    "referral_code": 0
                },
                "cctp": {
                    "value": 1000
                },
                "rebalancer": {
                    "value": 1000
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

    // Prepare Ethereum transaction
    let chain_id: u64 = 1;
    let nonce: u64 = 0x42;
    let gas_limit = 44386;
    let max_fee_per_gas = 0x4a817c800;
    let max_priority_fee_per_gas = 0x3b9aca00;
    let to_address = parse_eth_address("87870Bca3F3fD6335C3F4ce8392D69350B4fA4E2"); // Aave Lending Pool
    let value = 0;

    let empty_tx = EVMTransaction {
        chain_id,
        nonce,
        gas_limit,
        max_fee_per_gas,
        max_priority_fee_per_gas,
        to: Some(to_address),
        value,
        input: vec![],
        access_list: vec![],
    };

    let invest_args = json!({
        "destination_chain": 1,
        "aave_args": {
            "amount": 1000,
            "partial_transaction": empty_tx
        },
        "cctp_args": {
            "amount": 1000,
            "destination_domain": 100,
            "mint_recipient": address_to_bytes32_string("87870Bca3F3fD6335C3F4ce8392D69350B4fA4E2"),
            "burn_token": ("87870Bca3F3fD6335C3F4ce8392D69350B4fA4E2"),
            "destination_caller": address_to_bytes32_string("87870Bca3F3fD6335C3F4ce8392D69350B4fA4E2"),
            "max_fee": 0,
            "min_finality_threshold": 0,
            "message": [],
            "attestation": [],
            "partial_burn_transaction": empty_tx,
            "partial_mint_transaction": empty_tx
        },
        "rebalancer_args": {
            "amount": 1000,
            "source_chain": 1,
            "destination_chain": 1,
            "partial_transaction": empty_tx
        },
        "gas_invest": 10,
        "gas_cctp_burn": 10,
        "gas_cctp_mint": 40,
        "gas_aave": 20,
    });

    let invest_result = friendly_json_rpc_client
        .send_action(FunctionCallAction {
            method_name: "invest".to_string(),
            args: invest_args.to_string().into_bytes(), // Convert directly to Vec<u8>
            gas: 300000000000000,
            deposit: 0,
        })
        .await?;

    println!("Invest call result: {:?}", invest_result);

    let payloads = friendly_json_rpc_client
        .call_contract::<Vec<Vec<u8>>>(
            "get_signed_transactions",
            json!({
            "nonce": 1
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

    for (ptype, tx) in grouped {
        println!("Tx type {:?}: 0x{}", ptype, hex::encode(tx));
    }

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
