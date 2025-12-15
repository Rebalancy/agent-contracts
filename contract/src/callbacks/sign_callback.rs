use crate::{types::CacheKey, types::PayloadType, Contract, ContractExt};
use near_sdk::{env, near, require, PromiseError};
use omni_transaction::{
    evm::{types::Signature, EVMTransaction},
    signer::types::SignatureResponse,
};

#[near]
impl Contract {
    #[private]
    pub fn sign_callback(
        &mut self,
        #[callback_result] call_result: Result<SignatureResponse, PromiseError>,
        nonce: u64,
        tx_type: u8,
        ethereum_tx: EVMTransaction,
    ) -> Vec<u8> {
        // Ensure the callback corresponds to the active session.
        let nonce_from_session = self.get_active_session().nonce;
        require!(nonce == nonce_from_session, "Nonce mismatch in callback");

        let step =
            PayloadType::try_from(tx_type).unwrap_or_else(|_| env::panic_str("Unknown tx_type"));

        // Defense-in-depth: ensure correct order.
        self.assert_step_is_next(step);

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

                // payload: tx_type || signed_rlp
                let mut payload = vec![tx_type];
                payload.extend(signed_rlp.clone());

                // logs: update ActivityLog with the new signed transaction
                let mut log = self.logs.get(&nonce).expect("Log not found").clone();
                log.transactions.retain(|t| t[0] != tx_type);
                log.transactions.push(payload.clone());
                self.logs.insert(nonce, log);

                // caches: hash build_for_signing + signed payload
                let ph = self.hash_payload(&ethereum_tx); // [u8;32]
                let cache_key = CacheKey { nonce, tx_type };

                self.payload_hashes_by_nonce_and_type
                    .insert(cache_key.clone(), ph);

                self.signatures_by_nonce_and_type
                    .insert(cache_key, payload.clone());

                payload
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
