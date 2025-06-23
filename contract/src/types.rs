use borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::near;
use near_sdk::serde::{Deserialize, Serialize};
use omni_transaction::evm::EVMTransaction;
use schemars::JsonSchema;

pub type ChainId = u64;

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct Worker {
    pub checksum: String,
    pub codehash: String,
}

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
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Serialize, Deserialize, JsonSchema, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct CCTPConfig {
    pub value: u128,
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Serialize, Deserialize, JsonSchema, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct RebalancerConfig {
    pub value: u128,
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Serialize, Deserialize, JsonSchema, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Config {
    pub aave: AaveConfig,
    pub cctp: CCTPConfig,
    pub rebalancer: RebalancerConfig,
}

#[repr(u8)]
pub enum PayloadType {
    AaveSupply = 0,
    AaveWithdraw = 1,
    CCTPBurn = 2,
    CCTPMint = 3,
    RebalancerInvest = 4,
    RebalancerRebalance = 5,
}

// Activity Structs
#[derive(BorshDeserialize, BorshSerialize, Clone)]
pub struct ActivityLog {
    pub activity_type: String,
    pub source_chain: ChainId,
    pub destination_chain: ChainId,
    pub transactions: Vec<Vec<u8>>,
    pub timestamp: u64,
    pub nonce: u64,
}

// Args Structs
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AaveArgs {
    pub amount: u128,
    pub partial_transaction: EVMTransaction,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CCTPArgs {
    pub amount: u128,
    pub destination_domain: u32,
    pub mint_recipient: String,
    pub burn_token: String,
    pub destination_caller: String,
    pub max_fee: u128,
    pub min_finality_threshold: u32,
    pub message: Vec<u8>,
    pub attestation: Vec<u8>,
    pub partial_burn_transaction: EVMTransaction,
    pub partial_mint_transaction: EVMTransaction,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct RebalancerArgs {
    pub amount: u128,
    pub source_chain: ChainId,
    pub destination_chain: ChainId,
    pub partial_transaction: EVMTransaction,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct BuiltTx {
    pub encoded: Vec<u8>,
    pub tx: EVMTransaction,
}
