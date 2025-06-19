use near_workspaces::Account;
use omni_transaction::evm::{utils::parse_eth_address, EVMTransaction};
use serde_json::json;

#[tokio::test]
async fn test_invest() -> Result<(), Box<dyn std::error::Error>> {
    let worker = near_workspaces::sandbox().await?;

    let wasm_bytes =
        include_bytes!("../target/wasm32-unknown-unknown/release/shade_agent_contract.wasm")
            .to_vec();

    let contract = worker.dev_deploy(&wasm_bytes).await?;

    // Initialize contract
    let owner: Account = worker.root_account().unwrap();
    let init_result = contract
        .call("init")
        .args_json(json!({
            "owner_id": owner.id(),
            "source_chain": 1u64,
            "configs": [
                [1u64, {
                    "aave": {
                        "asset": "0x0000000000000000000000000000000000000000",
                        "on_behalf_of": "0x0000000000000000000000000000000000000000",
                        "referral_code": 0
                    },
                    "cctp": {},
                    "rebalancer": {}
                }]
            ]
        }))
        .transact()
        .await?
        .into_result()?;
    println!("Init result: {:?}", init_result);

    // Prepare Ethereum transaction
    let chain_id: u64 = 1;
    let nonce: u64 = 0x42;
    let gas_limit = 44386;
    let max_fee_per_gas = 0x4a817c800;
    let max_priority_fee_per_gas = 0x3b9aca00;
    let to_address = parse_eth_address("0x87870Bca3F3fD6335C3F4ce8392D69350B4fA4E2"); // Aave Lending Pool
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

    let result = contract
        .call("invest")
        .args_json(json!({
            "destination_chain": 1,
            "aave_args": {
                "amount": 1000,
                "partial_transaction": empty_tx
            },
            "cctp_args": {
                "amount": 1000,
                "destination_domain": 100,
                "mint_recipient": "0000000000000000000000000000000000000000000000000000000000000001",
                "burn_token": "0000000000000000000000000000000000000001",
                "destination_caller": "0000000000000000000000000000000000000001",
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
            }
        }))
        .transact()
        .await?;

    println!("Invest call result: {:?}", result);
    Ok(())
}
