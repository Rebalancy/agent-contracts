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
    CCTPApproveBeforeBurn = 2,
    CCTPBurn = 3,
    CCTPMint = 4,
    RebalancerWithdrawToAllocate = 5,
    RebalancerUpdateCrossChainBalance = 6,
    RebalancerDeposit = 7,
    RebalancerSignCrossChainBalance = 8,
}

impl From<u8> for PayloadType {
    fn from(value: u8) -> Self {
        match value {
            0 => PayloadType::AaveSupply,
            1 => PayloadType::AaveWithdraw,
            2 => PayloadType::CCTPApproveBeforeBurn,
            3 => PayloadType::CCTPBurn,
            4 => PayloadType::CCTPMint,
            5 => PayloadType::RebalancerWithdrawToAllocate,
            6 => PayloadType::RebalancerUpdateCrossChainBalance,
            7 => PayloadType::RebalancerDeposit,
            8 => PayloadType::RebalancerSignCrossChainBalance,
            _ => panic!("Unknown PayloadType: {}", value),
        }
    }
}
