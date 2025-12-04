use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use schemars::JsonSchema;

use crate::types::{AgentActionType, ChainId};

#[derive(BorshDeserialize, BorshSerialize, Clone, Serialize, Deserialize, JsonSchema, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ActivityLog {
    pub activity_type: AgentActionType,
    pub source_chain: ChainId,
    pub destination_chain: ChainId,
    pub timestamp: u64,
    pub nonce: u64,
    pub amount: u128,
    pub transactions: Vec<Vec<u8>>,
}
