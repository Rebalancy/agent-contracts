use borsh::BorshSchema;
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use schemars::JsonSchema;

use crate::types::PayloadType;

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

impl Flow {
    pub fn sequence(&self) -> &'static [PayloadType] {
        match self {
            Flow::AaveToAave => &[
                PayloadType::AaveWithdraw,
                PayloadType::CCTPBurn,
                PayloadType::CCTPMint,
                PayloadType::AaveSupply,
            ],
            Flow::RebalancerToAave => &[
                PayloadType::RebalancerWithdrawToAllocate,
                PayloadType::CCTPBurn,
                PayloadType::CCTPMint,
                PayloadType::AaveSupply,
            ],
            Flow::AaveToRebalancer => &[
                PayloadType::AaveWithdraw,
                PayloadType::CCTPBurn,
                PayloadType::CCTPMint,
                PayloadType::RebalancerDeposit,
            ],
        }
    }
}
