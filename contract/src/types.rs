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
    pub lending_pool_address: String,
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Serialize, Deserialize, JsonSchema, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct CCTPConfig {
    pub messenger_address: String,
    pub transmitter_address: String,
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

#[repr(u8)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    BorshDeserialize,
    BorshSerialize,
    JsonSchema,
)]
#[serde(crate = "near_sdk::serde")]
#[borsh(use_discriminant = true)]
pub enum AgentActionType {
    Rebalance,
    Harvest,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    JsonSchema,
    BorshDeserialize,
    BorshSerialize,
)]
#[serde(crate = "near_sdk::serde")]
pub enum Phase {
    AwaitingStartSignatures,
    StartCompleted,
    AwaitingCompleteSignatures,
    Finished,
}
impl From<AgentActionType> for u8 {
    fn from(action_type: AgentActionType) -> Self {
        match action_type {
            AgentActionType::Rebalance => 0,
            AgentActionType::Harvest => 1,
        }
    }
}

impl From<u8> for AgentActionType {
    fn from(value: u8) -> Self {
        match value {
            0 => AgentActionType::Rebalance,
            1 => AgentActionType::Harvest,
            _ => panic!("Unknown AgentActionType: {}", value),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum PayloadType {
    AaveSupply = 0,
    AaveWithdraw = 1,
    CCTPBurn = 2,
    CCTPMint = 3,
    RebalancerHarvest = 4,
    RebalancerInvest = 5,
}

impl From<u8> for PayloadType {
    fn from(value: u8) -> Self {
        match value {
            0 => PayloadType::AaveSupply,
            1 => PayloadType::AaveWithdraw,
            2 => PayloadType::CCTPBurn,
            3 => PayloadType::CCTPMint,
            4 => PayloadType::RebalancerHarvest,
            5 => PayloadType::RebalancerInvest,
            _ => panic!("Unknown PayloadType: {}", value),
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Serialize, Deserialize, JsonSchema, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ActivityLog {
    pub activity_type: AgentActionType,
    pub source_chain: ChainId,
    pub destination_chain: ChainId,
    pub timestamp: u64,
    pub nonce: u64,
    pub expected_amount: u128,
    pub actual_amount: Option<u128>,
    pub transactions: Vec<Vec<u8>>,
}

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

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct ActiveSession {
    pub nonce: u64,
    pub action_type: AgentActionType,
    pub started_at: u64,
    pub current_phase: Phase,
}
