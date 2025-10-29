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
    AaveApproveBeforeSupply = 2,
    CCTPApproveBeforeBurn = 3,
    CCTPBurn = 4,
    CCTPMint = 5,
    RebalancerWithdrawToAllocate = 6,
    RebalancerUpdateCrossChainBalance = 7,
    RebalancerDeposit = 8,
    RebalancerSignCrossChainBalance = 9,
}

impl From<u8> for PayloadType {
    fn from(value: u8) -> Self {
        match value {
            0 => PayloadType::AaveSupply,
            1 => PayloadType::AaveWithdraw,
            2 => PayloadType::AaveApproveBeforeSupply,
            3 => PayloadType::CCTPApproveBeforeBurn,
            4 => PayloadType::CCTPBurn,
            5 => PayloadType::CCTPMint,
            6 => PayloadType::RebalancerWithdrawToAllocate,
            7 => PayloadType::RebalancerUpdateCrossChainBalance,
            8 => PayloadType::RebalancerDeposit,
            9 => PayloadType::RebalancerSignCrossChainBalance,
            _ => panic!("Unknown PayloadType: {}", value),
        }
    }
}
