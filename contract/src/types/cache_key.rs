use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(
    BorshDeserialize,
    BorshSerialize,
    PartialEq,
    Eq,
    Hash,
    Ord,
    PartialOrd,
    Debug,
    Clone,
    Serialize,
    Deserialize,
    JsonSchema,
)]
#[serde(crate = "near_sdk::serde")]
pub struct CacheKey {
    pub nonce: u64,
    pub tx_type: u8,
}

impl CacheKey {
    pub fn new(nonce: u64, tx_type_u8: u8) -> Self {
        Self {
            nonce,
            tx_type: tx_type_u8,
        }
    }
}
