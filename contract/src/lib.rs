use dcap_qvl::verify;
use hex::{decode, encode};
use near_sdk::{
    env, near, require,
    store::{IterableMap, IterableSet},
    AccountId, NearToken, PanicOnDefault, Promise, PromiseError,
};
use omni_transaction::evm::types::Signature;
use omni_transaction::evm::EVMTransaction;
use omni_transaction::signer::types::{mpc_contract, SignRequest, SignatureResponse};

use constants::*;
use external::this_contract;
use types::Worker;

mod collateral;
mod constants;
mod external;
mod types;

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Contract {
    owner_id: AccountId,
    approved_codehashes: IterableSet<String>,
    worker_by_account_id: IterableMap<AccountId, Worker>,
}

#[near]
impl Contract {
    #[init]
    #[private]
    pub fn init(owner_id: AccountId) -> Self {
        Self {
            owner_id,
            approved_codehashes: IterableSet::new(b"a"),
            worker_by_account_id: IterableMap::new(b"b"),
        }
    }

    pub fn register_worker(
        &mut self,
        quote_hex: String,
        collateral: String,
        checksum: String,
        tcb_info: String,
    ) -> bool {
        let collateral = collateral::get_collateral(collateral);
        let quote = decode(quote_hex).unwrap();
        let now = env::block_timestamp() / 1000000000;
        let result = verify::verify(&quote, &collateral, now).expect("report is not verified");
        let rtmr3 = encode(result.report.as_td10().unwrap().rt_mr3.to_vec());
        let codehash = collateral::verify_codehash(tcb_info, rtmr3);

        // Only allow workers to register if their codehash is approved
        self.require_approved_codehash(&codehash);

        let predecessor = env::predecessor_account_id();
        self.worker_by_account_id
            .insert(predecessor, Worker { checksum, codehash });

        true
    }

    #[payable]
    pub fn hash_and_sign_evm_transaction(
        &mut self,
        evm_tx_params: EVMTransaction,
        attached_deposit: NearToken,
    ) -> Promise {
        // Only approve workers with approved codehashes can call this function
        self.require_worker_has_valid_codehash();

        let encoded_data = evm_tx_params.build_for_signing();

        let tx_hash = env::keccak256(&encoded_data);

        // Ensure the payload is exactly 32 bytes
        let payload: [u8; 32] = tx_hash.try_into().expect("Payload must be 32 bytes long");

        let request: SignRequest = SignRequest {
            payload,
            path: PATH.to_string(),
            key_version: KEY_VERSION,
        };

        let promise = mpc_contract::ext(MPC_CONTRACT_ACCOUNT_ID.parse().unwrap())
            .with_static_gas(GAS)
            .with_attached_deposit(attached_deposit)
            .sign(request);

        promise.then(
            this_contract::ext(env::current_account_id())
                .with_static_gas(CALLBACK_GAS)
                .callback(evm_tx_params),
        )
    }

    #[private]
    pub fn callback(
        &mut self,
        #[callback_result] call_result: Result<SignatureResponse, PromiseError>,
        ethereum_tx: EVMTransaction,
    ) -> String {
        match call_result {
            Ok(signature_response) => {
                env::log_str(&format!(
                    "Successfully received signature: big_r = {:?}, s = {:?}, recovery_id = {}",
                    signature_response.big_r, signature_response.s, signature_response.recovery_id
                ));

                // Extract r and s from the signature response
                let affine_point_bytes = hex::decode(signature_response.big_r.affine_point)
                    .expect("Failed to decode affine_point to bytes");

                env::log_str(&format!(
                    "Decoded affine_point bytes (length: {}): {:?}",
                    affine_point_bytes.len(),
                    hex::encode(&affine_point_bytes)
                ));

                // Extract r from the affine_point_bytes
                let r_bytes = affine_point_bytes[1..33].to_vec();
                assert_eq!(r_bytes.len(), 32, "r must be 32 bytes");

                env::log_str(&format!(
                    "Extracted r (32 bytes): {:?}",
                    hex::encode(&r_bytes)
                ));

                let s_bytes = hex::decode(signature_response.s.scalar)
                    .expect("Failed to decode scalar to bytes");

                assert_eq!(s_bytes.len(), 32, "s must be 32 bytes");

                env::log_str(&format!(
                    "Decoded s (32 bytes): {:?}",
                    hex::encode(&s_bytes)
                ));

                // decode the address from the signature response

                // Calculate v
                let v = signature_response.recovery_id as u64;
                env::log_str(&format!("Calculated v: {}", v));

                let signature_omni = Signature {
                    v,
                    r: r_bytes,
                    s: s_bytes,
                };
                let omni_evm_tx_encoded_with_signature =
                    ethereum_tx.build_with_signature(&signature_omni);

                env::log_str(&format!(
                    "Successfully signed transaction: {:?}",
                    omni_evm_tx_encoded_with_signature
                ));

                // Serialise the updated transaction
                hex::encode(omni_evm_tx_encoded_with_signature)
            }
            Err(error) => {
                env::log_str(&format!("Callback failed with error: {:?}", error));
                "Callback failed".to_string()
            }
        }
    }

    // Access control

    pub fn require_owner(&self) {
        require!(env::predecessor_account_id() == self.owner_id);
    }

    pub fn require_approved_codehash(&self, codehash: &String) {
        require!(self.approved_codehashes.contains(codehash));
    }

    pub fn require_worker_has_valid_codehash(&mut self) {
        let worker = self.get_worker(env::predecessor_account_id());
        require!(self.approved_codehashes.contains(&worker.codehash));
    }

    // Admin functions

    pub fn approve_codehash(&mut self, codehash: String) {
        self.require_owner();
        self.approved_codehashes.insert(codehash);
    }

    // Views

    pub fn get_worker(&self, account_id: AccountId) -> Worker {
        self.worker_by_account_id
            .get(&account_id)
            .unwrap()
            .to_owned()
    }
}
