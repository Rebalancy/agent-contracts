use near_sdk::{ext_contract, serde::Serialize};
use omni_transaction::evm::EVMTransaction;

#[derive(Debug, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SignRequest {
    pub payload: [u8; 32],
    pub path: String,
    pub key_version: u32,
}

#[allow(dead_code)]
#[ext_contract(mpc_contract)]
trait MPCContract {
    fn sign(&self, request: SignRequest);
}

#[allow(dead_code)]
#[ext_contract(this_contract)]
trait ThisContract {
    fn sign_callback(&self, nonce: u64, tx_type: u8, ethereum_tx: EVMTransaction) -> Vec<u8>;
}
