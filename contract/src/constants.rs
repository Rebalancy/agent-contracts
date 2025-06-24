use near_sdk::Gas;

pub const MPC_CONTRACT_ACCOUNT_ID: &str = "v1.signer-prod.testnet"; // PRODUCTION: "v1.signer"
pub const GAS: Gas = Gas::from_tgas(100);
pub const PATH: &str = "ethereum-1";
pub const KEY_VERSION: u32 = 0;
pub const CALLBACK_GAS: Gas = Gas::from_tgas(90);
pub const AAVE_LENDING_POOL_ADDRESS: &str = "0x7d2768de84f9a91b2c744cf0f0865d2e4b30f4bf"; // Mainnet Aave Lending Pool address
