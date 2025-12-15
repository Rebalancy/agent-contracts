use crate::{Contract, ContractExt};
use near_sdk::{env, near, require, PromiseError};
use omni_transaction::signer::types::SignatureResponse;

#[near]
impl Contract {
    #[private]
    pub fn sign_crosschain_balance_callback(
        &mut self,
        #[callback_result] call_result: Result<SignatureResponse, PromiseError>,
    ) -> Vec<u8> {
        match call_result {
            Ok(signature_response) => {
                // decode signature and build it into [u8;65]
                let affine_point_bytes =
                    hex::decode(signature_response.big_r.affine_point.clone()).expect("bad affine");
                require!(affine_point_bytes.len() >= 33, "affine too short");

                let r_bytes = affine_point_bytes[1..33].to_vec();
                let s_bytes = hex::decode(signature_response.s.scalar.clone()).expect("bad s");
                require!(s_bytes.len() == 32, "s len != 32");
                let v = signature_response.recovery_id as u8;

                let mut signature_bytes = Vec::with_capacity(65);
                signature_bytes.extend_from_slice(&r_bytes);
                signature_bytes.extend_from_slice(&s_bytes);
                signature_bytes.push(v);

                env::log_str(&format!("✅ MPC signature ready: v={}, r,s ok", v));

                signature_bytes
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
