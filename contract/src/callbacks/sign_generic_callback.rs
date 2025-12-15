use crate::{Contract, ContractExt};
use near_sdk::{env, near, require, PromiseError};
use omni_transaction::{
    evm::{types::Signature, EVMTransaction},
    signer::types::SignatureResponse,
};

#[near]
impl Contract {
    #[private]
    pub fn sign_generic_callback(
        &mut self,
        #[callback_result] call_result: Result<SignatureResponse, PromiseError>,
        ethereum_tx: EVMTransaction,
    ) -> Vec<u8> {
        match call_result {
            Ok(signature_response) => {
                // decode signature and build signed RLP
                let affine_point_bytes =
                    hex::decode(signature_response.big_r.affine_point.clone()).expect("bad affine");
                require!(affine_point_bytes.len() >= 33, "affine too short");

                let r_bytes = affine_point_bytes[1..33].to_vec();
                let s_bytes = hex::decode(signature_response.s.scalar.clone()).expect("bad s");
                require!(s_bytes.len() == 32, "s len != 32");
                let v = signature_response.recovery_id as u64;
                let signature_omni = Signature {
                    v,
                    r: r_bytes,
                    s: s_bytes,
                };
                let signed_rlp = ethereum_tx.build_with_signature(&signature_omni);

                signed_rlp // TODO: Cache signature hashes
            }
            Err(e) => {
                env::log_str(&format!("Callback failed: {:?}", e));
                vec![]
            }
        }
    }
}

#[cfg(test)]
mod maintests {
    use crate::test_helpers::*;
    use crate::types::*;
    use near_sdk::env;
}
