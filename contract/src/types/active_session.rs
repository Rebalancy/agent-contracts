use borsh::BorshSchema;
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use schemars::JsonSchema;

use crate::types::Flow;

#[derive(
    Debug, Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, BorshSchema,
)]
#[serde(crate = "near_sdk::serde")]
pub struct ActiveSession {
    pub nonce: u64,
    pub flow: Flow,
    pub started_at: u64,
}
