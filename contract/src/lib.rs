use std::str::FromStr;

use borsh::{BorshDeserialize, BorshSerialize};
use dcap_qvl::verify;
use hex::{decode, encode};
use near_sdk::{
    env, near, require,
    serde::{Deserialize, Serialize},
    store::{IterableMap, IterableSet},
    AccountId, NearToken, PanicOnDefault, Promise, PromiseError,
};
use omni_transaction::evm::types::Signature;
use omni_transaction::evm::EVMTransaction;
use omni_transaction::signer::types::SignatureResponse;

use alloy_primitives::{Address, B256, U256};
use constants::*;
use external::this_contract;
use schemars::JsonSchema;
use types::Worker;

mod collateral;
mod constants;
mod ecdsa;
mod encoders;
mod external;
mod types;

pub type ChainId = u64;

// Config Structs
#[derive(BorshDeserialize, BorshSerialize, Clone, Serialize, Deserialize, JsonSchema, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct AaveConfig {
    pub asset: String,
    pub on_behalf_of: String,
    pub referral_code: u16,
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Serialize, Deserialize, JsonSchema, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct CCTPConfig {
    pub value: u128,
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Serialize, Deserialize, JsonSchema, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct RebalancerConfig {
    pub value: u128,
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Serialize, Deserialize, JsonSchema, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Config {
    aave: AaveConfig,
    cctp: CCTPConfig,
    rebalancer: RebalancerConfig,
}

#[repr(u8)]
pub enum PayloadType {
    AaveSupply = 0,
    AaveWithdraw = 1,
    CCTPBurn = 2,
    CCTPMint = 3,
    RebalancerInvest = 4,
    RebalancerRebalance = 5,
}

// Activity Structs
#[derive(BorshDeserialize, BorshSerialize, Clone)]
pub struct ActivityLog {
    pub activity_type: String,
    pub source_chain: ChainId,
    pub destination_chain: ChainId,
    pub transactions: Vec<Vec<u8>>,
    pub timestamp: u64,
    pub nonce: u64,
}

// Args Structs
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct AaveArgs {
    pub amount: u128,
    pub partial_transaction: EVMTransaction,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct CCTPArgs {
    pub amount: u128,
    pub destination_domain: u32,
    pub mint_recipient: String,
    pub burn_token: String,
    pub destination_caller: String,
    pub max_fee: u128,
    pub min_finality_threshold: u32,
    pub message: Vec<u8>,
    pub attestation: Vec<u8>,
    pub partial_burn_transaction: EVMTransaction,
    pub partial_mint_transaction: EVMTransaction,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct RebalancerArgs {
    pub amount: u128,
    pub source_chain: ChainId,
    pub destination_chain: ChainId,
    pub partial_transaction: EVMTransaction,
}

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

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct ChainConfig {
    pub chain_id: ChainId,
    pub config: Config,
}

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

    /// Invest is an action that means:
    /// 1. Withdraw from the source chain vault
    /// 2. Bridge the withdrawn amount to the destination chain
    /// 3. Supply the bridged amount into Aave on the destination chain
    pub fn invest(
        &mut self,
        destination_chain: ChainId,
        aave_args: AaveArgs,
        cctp_args: CCTPArgs,
        rebalancer_args: RebalancerArgs,
    ) -> Promise {
        // TODO: validate that the caller is the shade agent

        // let source_chain_config = self
        //     .config
        //     .get(&self.source_chain)
        //     .expect("Chain not configured");

        // 1. Encode Vault Payload
        let invest_data = encoders::rebalancer::vault::encode_invest(rebalancer_args.amount);

        // Create invest transaction
        let mut invest_tx = rebalancer_args.partial_transaction;
        invest_tx.input = invest_data;

        let destination_chain_config = self
            .config
            .get(&destination_chain)
            .expect("Chain not configured");

        // 2.1 Encode Bridge Payload (Burn)
        let burn_usdc_data = encoders::cctp::messenger::encode_deposit_for_burn(
            U256::from(cctp_args.amount),
            cctp_args.destination_domain,
            B256::from_str(&cctp_args.mint_recipient).unwrap(),
            Address::from_str(&cctp_args.burn_token).unwrap(),
            B256::from_str(&cctp_args.destination_caller).unwrap(),
            U256::from(cctp_args.max_fee),
            cctp_args.min_finality_threshold,
        );

        // Create burn transaction
        let mut burn_tx = cctp_args.partial_burn_transaction;
        burn_tx.input = burn_usdc_data;

        // 2.2 Encode Bridge Payload (Mint)
        let mint_usdc_data = encoders::cctp::transmitter::encode_receive_message(
            cctp_args.message,
            cctp_args.attestation,
        );

        // Create mint transaction
        let mut mint_tx = cctp_args.partial_mint_transaction;
        mint_tx.input = mint_usdc_data;

        // 3. Encode Aave Supply Payload
        let aave_data = encoders::aave::lending_pool::encode_supply(
            Address::from_str(&destination_chain_config.aave.asset).unwrap(),
            U256::from(aave_args.amount),
            Address::from_str(&destination_chain_config.aave.on_behalf_of).unwrap(),
            destination_chain_config.aave.referral_code,
        );

        //  Create Aave supply transaction
        let mut aave_tx = aave_args.partial_transaction;
        aave_tx.input = aave_data;

        let nonce = self.logs_nonce;
        self.logs_nonce += 1;

        // 4. Log activity with nonce
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

        // 5. Manually create the promises for each payload
        let prom_rebalancer = ecdsa::get_sig(
            invest_tx.build_for_signing().try_into().unwrap(),
            "path_0".to_string(),
            KEY_VERSION,
        )
        .then(
            this_contract::ext(env::current_account_id())
                .with_static_gas(CALLBACK_GAS)
                .sign_callback(nonce, PayloadType::RebalancerInvest as u8, invest_tx),
        );

        let prom_burn = ecdsa::get_sig(
            burn_tx.build_for_signing().try_into().unwrap(),
            "path_1".to_string(),
            KEY_VERSION,
        )
        .then(
            this_contract::ext(env::current_account_id())
                .with_static_gas(CALLBACK_GAS)
                .sign_callback(nonce, PayloadType::CCTPBurn as u8, burn_tx),
        );

        let prom_mint = ecdsa::get_sig(
            mint_tx.build_for_signing().try_into().unwrap(),
            "path_2".to_string(),
            KEY_VERSION,
        )
        .then(
            this_contract::ext(env::current_account_id())
                .with_static_gas(CALLBACK_GAS)
                .sign_callback(nonce, PayloadType::CCTPMint as u8, mint_tx),
        );

        let prom_aave = ecdsa::get_sig(
            aave_tx.build_for_signing().try_into().unwrap(),
            "path_3".to_string(),
            KEY_VERSION,
        )
        .then(
            this_contract::ext(env::current_account_id())
                .with_static_gas(CALLBACK_GAS)
                .sign_callback(nonce, PayloadType::AaveSupply as u8, aave_tx),
        );

        // 6. Return the promises to be executed
        prom_rebalancer
            .and(prom_burn)
            .and(prom_mint)
            .and(prom_aave)
            .as_return()
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
            Ok(sig) => {
                let r_bytes = hex::decode(sig.big_r.affine_point).unwrap()[1..33].to_vec();
                let s_bytes = hex::decode(sig.s.scalar).unwrap();
                let signature = Signature {
                    v: sig.recovery_id as u64,
                    r: r_bytes,
                    s: s_bytes,
                };
                let signed_tx = ethereum_tx.build_with_signature(&signature);
                let mut payload = vec![tx_type];
                payload.extend(signed_tx);

                let mut log = self.logs.get(&nonce).expect("Log not found").clone();
                log.transactions.push(payload.clone());
                self.logs.insert(nonce, log);

                payload
            }
            Err(e) => {
                env::log_str(&format!("Signature callback failed: {:?}", e));
                vec![]
            }
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
