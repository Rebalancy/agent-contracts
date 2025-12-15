use borsh::BorshSchema;
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use schemars::JsonSchema;

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
    BorshSerialize,
    BorshDeserialize,
    JsonSchema,
    BorshSchema,
)]
#[serde(crate = "near_sdk::serde")]
#[borsh(use_discriminant = true)]
pub enum PayloadType {
    AaveSupply = 0,
    AaveWithdraw = 1,
    CCTPBurn = 2,
    CCTPMint = 3,
    RebalancerWithdrawToAllocate = 4,
    RebalancerUpdateCrossChainBalance = 5,
    RebalancerDeposit = 6,
    RebalancerSignCrossChainBalance = 7,
}

impl From<u8> for PayloadType {
    fn from(value: u8) -> Self {
        match value {
            0 => PayloadType::AaveSupply,
            1 => PayloadType::AaveWithdraw,
            2 => PayloadType::CCTPBurn,
            3 => PayloadType::CCTPMint,
            4 => PayloadType::RebalancerWithdrawToAllocate,
            5 => PayloadType::RebalancerUpdateCrossChainBalance,
            6 => PayloadType::RebalancerDeposit,
            7 => PayloadType::RebalancerSignCrossChainBalance,
            _ => panic!("Unknown PayloadType: {}", value),
        }
    }
}
