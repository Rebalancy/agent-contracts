use near_sdk::{Gas, NearToken};

pub const MPC_CONTRACT_ACCOUNT_ID: &str = "v1.signer";
pub const MPC_CONTRACT_ACCOUNT_ID_TESTNET: &str = "v1.signer-prod.testnet";
pub const PATH: &str = "ethereum-1";
pub const KEY_VERSION: u32 = 0;
pub const CALLBACK_GAS: Gas = Gas::from_tgas(50);
pub const OPERATIONS_TIMEOUT: u64 = 1 * 60 * 1000; // TODO: Restore before deploying 1 minutes timeout
pub const ATTACHED_DEPOSIT: NearToken = NearToken::from_yoctonear(500000000000000000000000);
