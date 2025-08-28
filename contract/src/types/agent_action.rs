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
    BorshDeserialize,
    BorshSerialize,
    JsonSchema,
    BorshSchema,
)]
#[serde(crate = "near_sdk::serde")]
#[borsh(use_discriminant = true)]
pub enum AgentActionType {
    Rebalance, // withdraw to allocate
    UpdateCrossChainBalance,
    Deposit,
    SignCrossChainBalance,
}

impl From<AgentActionType> for u8 {
    fn from(action_type: AgentActionType) -> Self {
        match action_type {
            AgentActionType::Rebalance => 0, // withdraw to allocate
            AgentActionType::UpdateCrossChainBalance => 1,
            AgentActionType::Deposit => 2,
            AgentActionType::SignCrossChainBalance => 3,
        }
    }
}

impl From<u8> for AgentActionType {
    fn from(value: u8) -> Self {
        match value {
            0 => AgentActionType::Rebalance,
            1 => AgentActionType::UpdateCrossChainBalance,
            2 => AgentActionType::Deposit,
            3 => AgentActionType::SignCrossChainBalance,
            _ => panic!("Unknown AgentActionType: {}", value),
        }
    }
}
