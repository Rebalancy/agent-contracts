use near_sdk::serde::{Deserialize, Serialize};
use omni_transaction::evm::EVMTransaction;
use schemars::JsonSchema;

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AaveArgs {
    pub amount: u128,
    pub partial_transaction: EVMTransaction,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CCTPMintArgs {
    pub message: Vec<u8>,
    pub attestation: Vec<u8>,
    pub partial_mint_transaction: EVMTransaction,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CCTPBeforeBurnArgs {
    pub spender: String,
    pub amount: u128,
    pub partial_transaction: EVMTransaction,
}

pub type AaveApproveBeforeSupplyArgs = CCTPBeforeBurnArgs;

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CCTPBurnArgs {
    pub amount: u128,
    pub destination_domain: u32,
    pub mint_recipient: String,
    pub burn_token: String,
    pub destination_caller: String,
    pub max_fee: u128,
    pub min_finality_threshold: u32,
    pub partial_burn_transaction: EVMTransaction,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct RebalancerArgs {
    pub amount: u128,
    pub partial_transaction: EVMTransaction,
    pub cross_chain_a_token_balance: Option<u128>,
}
