use std::str::FromStr;

use dcap_qvl::verify;
use hex::{decode, encode};
use near_sdk::{
    env, near, require,
    store::{IterableMap, IterableSet},
    AccountId, Gas, NearToken, PanicOnDefault, Promise, PromiseError,
};
use omni_transaction::evm::types::Signature;
use omni_transaction::evm::EVMTransaction;
use omni_transaction::signer::types::SignatureResponse;

use alloy_primitives::{Address, B256, U256};
use constants::*;
use external::this_contract;
use types::Worker;

use crate::types::{
    AaveArgs, ActivityLog, CCTPArgs, ChainConfig, ChainId, Config, PayloadType, RebalancerArgs,
};

mod collateral;
mod constants;
mod ecdsa;
mod encoders;
mod external;
mod tx_builders;
mod types;

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Contract {
    owner_id: AccountId,
    source_chain: ChainId,
    approved_codehashes: IterableSet<String>,
    worker_by_account_id: IterableMap<AccountId, Worker>,
    pub config: IterableMap<ChainId, Config>,
    pub logs: IterableMap<u64, ActivityLog>,
    pub logs_nonce: u64,
}

const PATH: &str = "rebalancer";

#[near]
impl Contract {
    #[init]
    #[private]
    pub fn init(source_chain: ChainId, configs: Vec<ChainConfig>) -> Self {
        let owner_id = env::predecessor_account_id();

        let mut contract = Self {
            owner_id,
            approved_codehashes: IterableSet::new(b"a"),
            worker_by_account_id: IterableMap::new(b"b"),
            config: IterableMap::new(b"c"),
            source_chain,
            logs: IterableMap::new(b"d"),
            logs_nonce: 0,
        };
        for cfg in configs {
            contract.config.insert(cfg.chain_id, cfg.config);
        }
        contract
    }

    // DeFi functions

    // Invest
    /// Invest is an action that means:
    /// 1. Withdraw from the source chain vault
    /// 2. Bridge the withdrawn amount to the destination chain
    /// 3. Supply the bridged amount into Aave on the destination chain
    ///
    pub fn invest(
        &mut self,
        destination_chain: ChainId,
        rebalancer_args: RebalancerArgs,
        cctp_args: CCTPArgs,
        aave_args: AaveArgs,
        gas_invest: u64,
        gas_cctp_burn: u64,
        gas_cctp_mint: u64,
        gas_aave: u64,
    ) -> u64 {
        // TODO: validate that the caller is the shade agent

        let nonce = self.logs_nonce;
        self.logs_nonce += 1;

        self.logs.insert(
            nonce,
            ActivityLog {
                activity_type: "invest".to_string(),
                source_chain: self.source_chain,
                destination_chain,
                transactions: vec![],
                timestamp: env::block_timestamp_ms(),
                nonce,
            },
        );

        self.build_invest_tx(rebalancer_args, nonce, gas_invest);
        self.build_cctp_burn_tx(cctp_args.clone(), nonce, gas_cctp_burn);
        self.build_cctp_mint_tx(cctp_args.clone(), nonce, gas_cctp_mint);
        self.build_aave_tx(destination_chain, aave_args, nonce, gas_aave);

        env::log_str(&format!("Invest started for nonce {}", nonce));
        nonce
    }

    fn build_invest_tx(&self, args: RebalancerArgs, nonce: u64, gas: u64) -> Promise {
        let callback_gas: Gas = Gas::from_tgas(gas);

        let input = encoders::rebalancer::vault::encode_invest(args.amount);
        let mut tx = args.partial_transaction;
        tx.input = input;

        let payload = self.hash_payload(&tx);

        ecdsa::get_sig(payload, PATH.to_string(), KEY_VERSION).then(
            this_contract::ext(env::current_account_id())
                .with_static_gas(callback_gas)
                .sign_callback(nonce, PayloadType::RebalancerInvest as u8, tx),
        )
    }

    fn build_cctp_burn_tx(&self, args: CCTPArgs, nonce: u64, gas: u64) -> Promise {
        let callback_gas: Gas = Gas::from_tgas(gas);

        let input = encoders::cctp::messenger::encode_deposit_for_burn(
            U256::from(args.amount),
            args.destination_domain,
            B256::from_str(&args.mint_recipient).expect("Invalid recipient"),
            Address::from_str(&args.burn_token).expect("Invalid token address"),
            B256::from_str(&args.destination_caller).expect("Invalid destination caller"),
            U256::from(args.max_fee),
            args.min_finality_threshold,
        );
        let mut tx = args.partial_burn_transaction;
        tx.input = input;

        let payload = self.hash_payload(&tx);

        ecdsa::get_sig(payload, PATH.to_string(), KEY_VERSION).then(
            this_contract::ext(env::current_account_id())
                .with_static_gas(callback_gas)
                .sign_callback(nonce, PayloadType::CCTPBurn as u8, tx),
        )
    }

    fn build_cctp_mint_tx(&self, args: CCTPArgs, nonce: u64, gas: u64) -> Promise {
        let callback_gas: Gas = Gas::from_tgas(gas);

        let input = encoders::cctp::transmitter::encode_receive_message(
            args.message.clone(),
            args.attestation.clone(),
        );
        let mut tx = args.partial_mint_transaction;
        tx.input = input;

        let payload = self.hash_payload(&tx);

        ecdsa::get_sig(payload, PATH.to_string(), KEY_VERSION).then(
            this_contract::ext(env::current_account_id())
                .with_static_gas(callback_gas)
                .sign_callback(nonce, PayloadType::CCTPMint as u8, tx),
        )
    }

    fn build_aave_tx(
        &self,
        destination_chain: ChainId,
        args: AaveArgs,
        nonce: u64,
        gas: u64,
    ) -> Promise {
        let callback_gas: Gas = Gas::from_tgas(gas);

        let destination_chain_config = self
            .config
            .get(&destination_chain)
            .expect("Chain not configured");

        let input = encoders::aave::lending_pool::encode_supply(
            Address::from_str(&destination_chain_config.aave.asset).expect("Invalid asset address"),
            U256::from(args.amount),
            Address::from_str(&destination_chain_config.aave.on_behalf_of)
                .expect("Invalid on_behalf_of address"),
            destination_chain_config.aave.referral_code,
        );
        let mut tx = args.partial_transaction;
        tx.input = input;

        let payload = self.hash_payload(&tx);

        ecdsa::get_sig(payload, PATH.to_string(), KEY_VERSION).then(
            this_contract::ext(env::current_account_id())
                .with_static_gas(callback_gas)
                .sign_callback(nonce, PayloadType::AaveSupply as u8, tx),
        )
    }

    #[private]
    pub fn sign_callback(
        &mut self,
        #[callback_result] call_result: Result<SignatureResponse, PromiseError>,
        nonce: u64,
        tx_type: u8,
        ethereum_tx: EVMTransaction,
    ) -> Vec<u8> {
        match call_result {
            Ok(signature_response) => {
                env::log_str(&format!(
                    "Successfully received signature: big_r = {:?}, s = {:?}, recovery_id = {}",
                    signature_response.big_r, signature_response.s, signature_response.recovery_id
                ));

                // Extract r and s from the signature response
                let affine_point_bytes =
                    match hex::decode(signature_response.big_r.affine_point.clone()) {
                        Ok(bytes) => bytes,
                        Err(e) => {
                            env::log_str(&format!(
                                "Failed to decode affine_point to bytes: {:?}",
                                e
                            ));
                            return vec![];
                        }
                    };

                env::log_str(&format!(
                    "Decoded affine_point bytes (length: {}): {:?}",
                    affine_point_bytes.len(),
                    hex::encode(&affine_point_bytes)
                ));

                if affine_point_bytes.len() < 33 {
                    env::log_str("Affine point bytes too short to extract r.");
                    return vec![];
                }

                // Extract r from the affine_point_bytes
                let r_bytes = affine_point_bytes[1..33].to_vec();
                assert_eq!(r_bytes.len(), 32, "r must be 32 bytes");

                env::log_str(&format!(
                    "Extracted r (32 bytes): {:?}",
                    hex::encode(&r_bytes)
                ));

                let s_bytes = match hex::decode(signature_response.s.scalar.clone()) {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        env::log_str(&format!("Failed to decode scalar to bytes: {:?}", e));
                        return vec![];
                    }
                };

                if s_bytes.len() != 32 {
                    env::log_str(&format!("Decoded s has invalid length: {}", s_bytes.len()));
                    return vec![];
                }

                env::log_str(&format!(
                    "Decoded s (32 bytes): {:?}",
                    hex::encode(&s_bytes)
                ));

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
                let mut payload = vec![tx_type];
                payload.extend(omni_evm_tx_encoded_with_signature);

                // Log and store the transaction in self.logs
                let mut log = self.logs.get(&nonce).expect("Log not found").clone();
                log.transactions.push(payload.clone());
                self.logs.insert(nonce, log);

                payload
            }
            Err(error) => {
                env::log_str(&format!("Callback failed with error: {:?}", error));
                vec![]
            }
        }
    }

    // Private functions
    fn hash_payload(&self, ethereum_tx: &EVMTransaction) -> [u8; 32] {
        env::keccak256(&ethereum_tx.build_for_signing())
            .try_into()
            .expect("Payload must be 32 bytes long")
    }

    // Agent functions
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

    pub fn get_signed_transactions(&self, nonce: u64) -> Vec<Vec<u8>> {
        self.logs
            .get(&nonce)
            .map(|log| log.transactions.clone())
            .unwrap_or_default()
    }

    pub fn get_worker(&self, account_id: AccountId) -> Worker {
        self.worker_by_account_id
            .get(&account_id)
            .unwrap()
            .to_owned()
    }
}
