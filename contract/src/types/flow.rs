use borsh::BorshSchema;
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(
    Clone,
    PartialEq,
    Eq,
    Debug,
    BorshDeserialize,
    BorshSerialize,
    Serialize,
    Deserialize,
    JsonSchema,
    BorshSchema,
)]
#[serde(crate = "near_sdk::serde")]
pub enum Flow {
    AaveToAave,
    RebalancerToAave,
    AaveToRebalancer,
}
