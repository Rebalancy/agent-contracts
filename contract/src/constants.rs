use near_sdk::Gas;

pub const MPC_CONTRACT_ACCOUNT_ID: &str = "v1.signer-prod.testnet"; // PRODUCTION: "v1.signer"
pub const GAS: Gas = Gas::from_tgas(50);
pub const PATH: &str = "ethereum-1";
pub const KEY_VERSION: u32 = 0;
pub const CALLBACK_GAS: Gas = Gas::from_tgas(200);
