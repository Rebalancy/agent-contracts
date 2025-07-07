use std::str::FromStr;

use dcap_qvl::verify;
use hex::{decode, encode};
use near_sdk::{
    env, near, require,
    store::{IterableMap, IterableSet, LookupMap},
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
    AaveArgs, ActiveSession, ActivityLog, AgentActionType, CCTPArgs, ChainConfig, ChainId, Config,
    PayloadType, Phase, RebalancerArgs,
};

mod collateral;
mod constants;
mod ecdsa;
mod encoders;
mod external;
mod tx_builders;
pub mod types;

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Contract {
    owner_id: AccountId,
    source_chain: ChainId,
    approved_codehashes: IterableSet<String>,
    worker_by_account_id: IterableMap<AccountId, Worker>,
    pub config: LookupMap<ChainId, Config>,
    pub logs: IterableMap<u64, ActivityLog>,
    pub logs_nonce: u64,
    pub allocations: LookupMap<ChainId, u128>,
    pub supported_chains: Vec<ChainId>,
    pub active_session: Option<ActiveSession>,
}

const PATH: &str = "rebalancer";
const OPERATIONS_TIMEOUT: u64 = 5 * 60 * 1000; // 5 minutes timeout

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
            config: LookupMap::new(b"c"),
            source_chain,
            allocations: LookupMap::new(b"d"),
            logs: IterableMap::new(b"e"),
            logs_nonce: 0,
            active_session: None,
            supported_chains: configs.iter().map(|cfg| cfg.chain_id.clone()).collect(),
        };
        for cfg in configs {
            contract.config.insert(cfg.chain_id, cfg.config);
        }
        contract
    }

    pub fn add_supported_chain(&mut self, new_chain: ChainConfig) {
        self.require_owner();
        require!(
            !self.supported_chains.contains(&new_chain.chain_id),
            "Chain already supported"
        );
        self.supported_chains.push(new_chain.chain_id.clone());
        self.config.insert(new_chain.chain_id, new_chain.config);
    }

    /*
    This function is called by the shade agent to start the rebalancing process.
    It will withdraw the specified amount from the source chain vault and bridge it to the destination chain
    using CCTP. The function takes the following parameters:
        - destination_chain: The chain where the funds will be bridged
        - rebalancer_args: Arguments for the rebalancer transaction
        - cctp_args: Arguments for the CCTP burn transaction
        - gas_invest: Gas limit for the rebalancer transaction
        - gas_cctp_burn: Gas limit for the CCTP burn transaction
    */
    pub fn start_rebalance(
        &mut self,
        source_chain: ChainId,
        destination_chain: ChainId,
        rebalancer_args: RebalancerArgs,
        cctp_args: CCTPArgs,
        gas_for_rebalancer: u64,
        gas_for_cctp_burn: u64,
    ) -> u64 {
        self.assert_no_active_session();

        // TODO: validate that the caller is the shade agent
        // TODO: validate the source and destination chains are supported

        let nonce = self.logs_nonce;
        self.logs_nonce += 1;

        let amount = rebalancer_args.amount;

        self.logs.insert(
            nonce,
            ActivityLog {
                activity_type: AgentActionType::Rebalance,
                source_chain: source_chain,
                destination_chain,
                transactions: vec![],
                timestamp: env::block_timestamp_ms(),
                nonce,
                expected_amount: amount,
                actual_amount: None,
            },
        );

        self.active_session = Some(ActiveSession {
            nonce,
            action_type: AgentActionType::Rebalance,
            started_at: env::block_timestamp_ms(),
            current_phase: Phase::AwaitingStartSignatures,
        });

        self.build_invest_rebalancer_tx(
            destination_chain,
            rebalancer_args,
            nonce,
            gas_for_rebalancer,
        );

        self.build_cctp_burn_tx(
            destination_chain,
            cctp_args.clone(),
            nonce,
            gas_for_cctp_burn,
        );

        env::log_str(&format!(
            "Rebalance start-phase initiated (nonce {})",
            nonce
        ));
        nonce
    }

    /*
    Complete the rebalancing process
    This function is called after the CCTP burn transaction has been confirmed on the source chain.
    It will mint the bridged amount on the destination chain and supply it into Aave.
    The function takes the following parameters:
        - destination_chain: The chain where the funds will be minted and supplied
        - cctp_args: Arguments for the CCTP mint transaction
        - aave_args: Arguments for the Aave supply transaction
        - gas_cctp_mint: Gas limit for the CCTP mint transaction
        - gas_aave: Gas limit for the Aave supply transaction
    */
    pub fn complete_rebalance(
        &mut self,
        cctp_args: CCTPArgs,
        aave_args: AaveArgs,
        gas_cctp_mint: u64,
        gas_aave: u64,
    ) {
        // TODO: validate that the caller is the shade agent

        let session = self.active_session.as_mut().expect("No active session");

        require!(
            session.action_type == AgentActionType::Rebalance,
            "Invalid session"
        );
        require!(
            session.current_phase == Phase::StartCompleted,
            "Start phase not completed"
        );

        session.current_phase = Phase::AwaitingCompleteSignatures;
        let nonce = session.nonce;
        let log = self.logs.get(&nonce).expect("Log not found").clone();

        self.build_cctp_mint_tx(
            log.destination_chain.clone(),
            cctp_args.clone(),
            nonce,
            gas_cctp_mint,
        );

        let aave_args_amount = aave_args.amount as u128;

        self.build_aave_supply_tx(log.destination_chain, aave_args, nonce, gas_aave);

        // Update allocations
        let current_destination_chain_allocation = self
            .allocations
            .get(&log.destination_chain)
            .cloned()
            .unwrap_or(0);

        self.allocations.insert(
            log.destination_chain.clone(),
            current_destination_chain_allocation + aave_args_amount,
        );

        let current_source_chain_destination = self
            .allocations
            .get(&log.source_chain)
            .cloned()
            .unwrap_or(0);

        self.allocations.insert(
            log.source_chain.clone(),
            current_source_chain_destination.saturating_sub(aave_args_amount),
        );

        env::log_str(&format!(
            "Rebalance complete-phase initiated (nonce {})",
            nonce
        ));
    }

    fn build_invest_rebalancer_tx(
        &self,
        destination_chain: ChainId,
        args: RebalancerArgs,
        nonce: u64,
        gas: u64,
    ) -> Promise {
        let callback_gas: Gas = Gas::from_tgas(gas);

        let destination_chain_config = self.get_chain_config(&destination_chain);

        let input = encoders::rebalancer::vault::encode_invest(args.amount);
        let mut tx = args.partial_transaction;
        tx.input = input;

        let address = Address::from_str(&destination_chain_config.rebalancer.vault_address)
            .expect("Invalid address string")
            .into_array();

        tx.to = Some(address);

        let payload = self.hash_payload(&tx);

        ecdsa::get_sig(payload, PATH.to_string(), KEY_VERSION).then(
            this_contract::ext(env::current_account_id())
                .with_static_gas(callback_gas)
                .sign_callback(nonce, PayloadType::RebalancerInvest as u8, tx),
        )
    }

    fn build_cctp_burn_tx(
        &self,
        destination_chain: ChainId,
        args: CCTPArgs,
        nonce: u64,
        gas: u64,
    ) -> Promise {
        let callback_gas: Gas = Gas::from_tgas(gas);

        let destination_chain_config = self.get_chain_config(&destination_chain);

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

        let address = Address::from_str(&destination_chain_config.cctp.messenger_address)
            .expect("Invalid address string")
            .into_array();

        tx.to = Some(address);

        let payload = self.hash_payload(&tx);

        ecdsa::get_sig(payload, PATH.to_string(), KEY_VERSION).then(
            this_contract::ext(env::current_account_id())
                .with_static_gas(callback_gas)
                .sign_callback(nonce, PayloadType::CCTPBurn as u8, tx),
        )
    }

    fn build_cctp_mint_tx(
        &self,
        destination_chain: ChainId,
        args: CCTPArgs,
        nonce: u64,
        gas: u64,
    ) -> Promise {
        let callback_gas: Gas = Gas::from_tgas(gas);

        let destination_chain_config = self.get_chain_config(&destination_chain);

        let input = encoders::cctp::transmitter::encode_receive_message(
            args.message.clone(),
            args.attestation.clone(),
        );
        let mut tx = args.partial_mint_transaction;
        tx.input = input;

        let address = Address::from_str(&destination_chain_config.cctp.transmitter_address)
            .expect("Invalid address string")
            .into_array();

        tx.to = Some(address);

        let payload = self.hash_payload(&tx);

        ecdsa::get_sig(payload, PATH.to_string(), KEY_VERSION).then(
            this_contract::ext(env::current_account_id())
                .with_static_gas(callback_gas)
                .sign_callback(nonce, PayloadType::CCTPMint as u8, tx),
        )
    }

    fn build_aave_supply_tx(
        &self,
        destination_chain: ChainId,
        args: AaveArgs,
        nonce: u64,
        gas: u64,
    ) -> Promise {
        let callback_gas: Gas = Gas::from_tgas(gas);

        let destination_chain_config = self.get_chain_config(&destination_chain);

        let input = encoders::aave::lending_pool::encode_supply(
            Address::from_str(&destination_chain_config.aave.asset).expect("Invalid asset address"),
            U256::from(args.amount),
            Address::from_str(&destination_chain_config.aave.on_behalf_of)
                .expect("Invalid on_behalf_of address"),
            destination_chain_config.aave.referral_code,
        );
        let mut tx = args.partial_transaction;
        tx.input = input;

        let address = Address::from_str(&destination_chain_config.aave.lending_pool_address)
            .expect("Invalid address string")
            .into_array();

        tx.to = Some(address);

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
        // First, extract needed session info and drop mutable borrow
        let (initial_phase, session_nonce) = {
            let session = self.active_session.as_ref().expect("No active session");
            (session.current_phase.clone(), session.nonce)
        };
        require!(session_nonce == nonce, "Nonce mismatch");
        // Validate expected tx_type for this phase
        let expected = match initial_phase {
            Phase::AwaitingStartSignatures => Self::expected_start_txs(),
            Phase::AwaitingCompleteSignatures => Self::expected_complete_txs(),
            _ => &[],
        };
        require!(
            expected.iter().any(|pt| *pt as u8 == tx_type),
            "Unexpected tx_type"
        );

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

                // Phase advancement
                // Now reborrow mutably to advance state if needed
                if initial_phase == Phase::AwaitingStartSignatures
                    && self.is_phase_fulfilled(Phase::AwaitingStartSignatures)
                {
                    let session = self.active_session.as_mut().unwrap();
                    session.current_phase = Phase::StartCompleted;
                    env::log_str("Start phase completed");
                } else if initial_phase == Phase::AwaitingCompleteSignatures
                    && self.is_phase_fulfilled(Phase::AwaitingCompleteSignatures)
                {
                    // final completion: clear session
                    self.active_session = None;
                    env::log_str("Rebalance finished and cleared");
                }

                payload
            }
            Err(error) => {
                env::log_str(&format!("Callback failed with error: {:?}", error));
                vec![]
            }
        }
    }

    // Helper functions
    fn hash_payload(&self, ethereum_tx: &EVMTransaction) -> [u8; 32] {
        env::keccak256(&ethereum_tx.build_for_signing())
            .try_into()
            .expect("Payload must be 32 bytes long")
    }

    pub fn get_chain_config(&self, destination_chain: &ChainId) -> &Config {
        self.config
            .get(destination_chain)
            .expect("Chain not configured")
    }

    fn clear_if_timed_out(&mut self) {
        if let Some(session) = &self.active_session {
            if env::block_timestamp_ms() - session.started_at > OPERATIONS_TIMEOUT {
                self.logs.remove(&session.nonce);
                self.active_session = None;
                env::log_str("Session timed out and cleared");
            }
        }
    }

    fn assert_no_active_session(&mut self) {
        self.clear_if_timed_out();
        require!(self.active_session.is_none(), "Another action in progress");
    }

    pub fn assert_agent_is_calling(&self) {
        self.require_worker_has_valid_codehash();
    }

    fn expected_start_txs() -> &'static [PayloadType] {
        &[PayloadType::RebalancerInvest, PayloadType::CCTPBurn]
    }

    fn expected_complete_txs() -> &'static [PayloadType] {
        &[PayloadType::CCTPMint, PayloadType::AaveSupply]
    }

    fn is_phase_fulfilled(&self, phase: Phase) -> bool {
        if let Some(session) = &self.active_session {
            let expected = match phase {
                Phase::AwaitingStartSignatures => Self::expected_start_txs(),
                Phase::AwaitingCompleteSignatures => Self::expected_complete_txs(),
                _ => return false,
            };
            let log = self.logs.get(&session.nonce).expect("Log not found");
            let signed: Vec<u8> = log.transactions.iter().map(|tx| tx[0]).collect();
            expected.iter().all(|pt| signed.contains(&(*pt as u8)))
        } else {
            false
        }
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

    pub fn require_worker_has_valid_codehash(&self) {
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
            .cloned()
            .expect("Worker not registered")
    }

    pub fn get_latest_logs(&self, count: u64) -> Vec<ActivityLog> {
        let mut logs = Vec::new();
        let current_nonce = self.logs_nonce;

        let start = if current_nonce > count {
            current_nonce - count
        } else {
            0
        };

        for nonce in (start..current_nonce).rev() {
            if let Some(log) = self.logs.get(&nonce) {
                logs.push(log.clone());
            }
        }

        logs
    }

    pub fn get_allocations(&self) -> Vec<(ChainId, u128)> {
        self.supported_chains
            .iter()
            .map(|chain_id| {
                let allocation = self.allocations.get(chain_id).cloned().unwrap_or(0);
                (chain_id.clone(), allocation)
            })
            .collect()
    }

    pub fn get_all_configs(&self) -> Vec<(ChainId, Config)> {
        self.supported_chains
            .iter()
            .filter_map(|chain_id| {
                self.config
                    .get(chain_id)
                    .map(|cfg| (chain_id.clone(), cfg.clone()))
            })
            .collect()
    }
}
