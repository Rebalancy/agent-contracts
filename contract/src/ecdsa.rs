use crate::{
    constants::{
        ATTACHED_DEPOSIT, CALLBACK_GAS, MPC_CONTRACT_ACCOUNT_ID, MPC_CONTRACT_ACCOUNT_ID_TESTNET,
    },
    *,
};
use external::{mpc_contract, SignRequest};
use near_sdk::Promise;

pub fn get_sig(payload: [u8; 32], path: String, key_version: u32) -> Promise {
    let request = SignRequest {
        payload,
        path,
        key_version,
    };

    let mpc_contract_id = if env::current_account_id().as_str().contains("testnet") {
        MPC_CONTRACT_ACCOUNT_ID_TESTNET
    } else {
        MPC_CONTRACT_ACCOUNT_ID
    };

    mpc_contract::ext(mpc_contract_id.parse().unwrap())
        .with_static_gas(CALLBACK_GAS)
        .with_attached_deposit(ATTACHED_DEPOSIT)
        .sign(request)
}
