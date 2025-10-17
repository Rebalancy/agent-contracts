use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use schemars::JsonSchema;

use crate::types::ChainId;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct ChainConfig {
    pub chain_id: ChainId,
    pub config: Config,
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Serialize, Deserialize, JsonSchema, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct AaveConfig {
    pub asset: String,
    pub on_behalf_of: String,
    pub referral_code: u16,
    pub lending_pool_address: String,
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Serialize, Deserialize, JsonSchema, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct CCTPConfig {
    pub messenger_address: String,
    pub transmitter_address: String,
    pub usdc_address: String,
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Serialize, Deserialize, JsonSchema, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct RebalancerConfig {
    pub vault_address: String,
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Serialize, Deserialize, JsonSchema, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Config {
    pub aave: AaveConfig,
    pub cctp: CCTPConfig,
    pub rebalancer: RebalancerConfig,
}
