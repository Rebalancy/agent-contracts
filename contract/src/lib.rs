use std::str::FromStr;

use crate::{
    constants::{KEY_VERSION, PATH},
    external::this_contract,
    types::{
        AaveApproveBeforeSupplyArgs, AaveArgs, ActiveSession, ActivityLog, AgentActionType,
        CCTPBeforeBurnArgs, CCTPBurnArgs, CCTPMintArgs, CacheKey, ChainConfig, ChainId, Config,
        Flow, PayloadType, RebalancerArgs, SnapshotDigestArgs, Step, Worker,
    },
};
use alloy_primitives::Address;
use near_sdk::{
    env, near, require,
    store::{IterableMap, IterableSet, LookupMap},
    AccountId, Gas, PanicOnDefault, Promise, PromiseError,
};
use omni_transaction::{
    evm::{types::Signature, EVMTransaction},
    signer::types::SignatureResponse,
};

mod access_control;
mod admin;
mod agent;
mod collateral;
mod constants;
mod ecdsa;
mod encoders;
mod external;
mod state_machine;
mod tx_builders;
mod views;

pub mod types;

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Contract {
    pub owner_id: AccountId,
    pub source_chain: ChainId,
    pub approved_codehashes: IterableSet<String>,
    pub worker_by_account_id: IterableMap<AccountId, Worker>,
    pub config: LookupMap<ChainId, Config>,
    pub logs: IterableMap<u64, ActivityLog>,
    pub logs_nonce: u64,
    pub allocations: LookupMap<ChainId, u128>,
    pub supported_chains: Vec<ChainId>,
    pub active_session: Option<ActiveSession>,
    pub signatures_by_nonce_and_type: LookupMap<CacheKey, Vec<u8>>, // (nonce, tx_type) -> signed RLP prefixed (tx_type || rlp)
    pub payload_hashes_by_nonce_and_type: LookupMap<CacheKey, [u8; 32]>, // (nonce, tx_type) -> payload_hash (build_for_signing)
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
            config: LookupMap::new(b"c"),
            source_chain,
            allocations: LookupMap::new(b"d"),
            logs: IterableMap::new(b"e"),
            logs_nonce: 0,
            active_session: None,
            supported_chains: configs.iter().map(|cfg| cfg.chain_id.clone()).collect(),
            signatures_by_nonce_and_type: LookupMap::new(b"f"),
            payload_hashes_by_nonce_and_type: LookupMap::new(b"g"),
        };
        for cfg in configs {
            contract.config.insert(cfg.chain_id, cfg.config);
        }
        contract
    }

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

#[near]
impl Contract {
    pub fn start_rebalance(
        &mut self,
        flow: Flow,
        source_chain: ChainId,
        destination_chain: ChainId,
        expected_amount: u128,
    ) -> u64 {
        self.assert_no_active_session();
        self.assert_agent_is_calling();
        self.is_chain_supported(&source_chain);
        self.is_chain_supported(&destination_chain);

        let nonce = self.logs_nonce;
        self.logs_nonce += 1;

        self.logs.insert(
            nonce,
            ActivityLog {
                activity_type: AgentActionType::Rebalance,
                source_chain: source_chain,
                destination_chain,
                transactions: vec![],
                timestamp: env::block_timestamp_ms(),
                nonce,
                expected_amount,
                actual_amount: None,
            },
        );

        self.active_session = Some(ActiveSession {
            nonce,
            flow,
            started_at: env::block_timestamp_ms(),
        });

        nonce
    }

    pub fn build_and_sign_withdraw_for_crosschain_allocation_tx(
        &mut self,
        rebalancer_args: RebalancerArgs,
        callback_gas_tgas: u64,
    ) -> Promise {
        self.assert_agent_is_calling();
        let cfg =
            self.get_chain_config_from_step_and_current_session(Step::RebalancerWithdrawToAllocate);

        let mut tx = rebalancer_args.clone().partial_transaction;
        tx.input = tx_builders::build_withdraw_for_crosschain_allocation_tx(rebalancer_args);
        tx.to = Some(
            Address::from_str(&cfg.rebalancer.vault_address)
                .expect("Invalid vault")
                .into_array(),
        );

        self.trigger_signature(Step::RebalancerWithdrawToAllocate, tx, callback_gas_tgas)
    }

    pub fn build_and_sign_cctp_approve_before_burn_tx(
        &mut self,
        args: CCTPBeforeBurnArgs,
        callback_gas_tgas: u64,
    ) -> Promise {
        self.assert_agent_is_calling();
        let cfg = self.get_chain_config_from_step_and_current_session(Step::CCTPApproveBeforeBurn);

        let mut tx = args.clone().partial_transaction;
        tx.input = tx_builders::build_cctp_approve_before_burn_tx(args);
        tx.to = Some(
            Address::from_str(&cfg.cctp.usdc_address)
                .expect("Invalid USDC address")
                .into_array(),
        );

        self.trigger_signature(Step::CCTPApproveBeforeBurn, tx, callback_gas_tgas)
    }

    pub fn build_and_sign_cctp_burn_tx(
        &mut self,
        args: CCTPBurnArgs,
        callback_gas_tgas: u64,
    ) -> Promise {
        self.assert_agent_is_calling();
        let cfg = self.get_chain_config_from_step_and_current_session(Step::CCTPBurn);

        let mut tx = args.clone().partial_burn_transaction;
        tx.input = tx_builders::build_cctp_burn_tx(args);
        tx.to = Some(
            Address::from_str(&cfg.cctp.messenger_address)
                .expect("Invalid messenger")
                .into_array(),
        );

        self.trigger_signature(Step::CCTPBurn, tx, callback_gas_tgas)
    }

    pub fn build_and_sign_cctp_mint_tx(
        &mut self,
        args: CCTPMintArgs,
        callback_gas_tgas: u64,
    ) -> Promise {
        self.assert_agent_is_calling();
        let cfg = self.get_chain_config_from_step_and_current_session(Step::CCTPMint);

        let mut tx = args.clone().partial_mint_transaction;
        tx.input = tx_builders::build_cctp_mint_tx(args);
        tx.to = Some(
            Address::from_str(&cfg.cctp.transmitter_address)
                .expect("Invalid transmitter")
                .into_array(),
        );

        self.trigger_signature(Step::CCTPMint, tx, callback_gas_tgas)
    }

    pub fn build_and_sign_aave_approve_before_supply_tx(
        &mut self,
        args: AaveApproveBeforeSupplyArgs,
        callback_gas_tgas: u64,
    ) -> Promise {
        self.assert_agent_is_calling();
        let cfg =
            self.get_chain_config_from_step_and_current_session(Step::AaveApproveBeforeSupply);

        let mut tx = args.clone().partial_transaction;
        tx.input = tx_builders::build_aave_approve_before_supply_tx(args);
        tx.to = Some(
            Address::from_str(&cfg.cctp.usdc_address)
                .expect("Invalid USDC address")
                .into_array(),
        );

        self.trigger_signature(Step::AaveApproveBeforeSupply, tx, callback_gas_tgas)
    }

    pub fn build_and_sign_aave_supply_tx(
        &mut self,
        args: AaveArgs,
        callback_gas_tgas: u64,
    ) -> Promise {
        self.assert_agent_is_calling();
        self.assert_step_is_next(Step::AaveSupply);

        let cfg = self.get_chain_config_from_step_and_current_session(Step::AaveSupply);

        let mut tx = args.clone().partial_transaction;
        tx.input = tx_builders::build_aave_supply_tx(args, cfg.aave.clone());
        tx.to = Some(
            Address::from_str(&cfg.aave.lending_pool_address)
                .expect("Invalid lending pool")
                .into_array(),
        );

        self.trigger_signature(Step::AaveSupply, tx, callback_gas_tgas)
    }

    pub fn build_and_sign_aave_withdraw_tx(
        &mut self,
        args: AaveArgs,
        callback_gas_tgas: u64,
    ) -> Promise {
        self.assert_agent_is_calling();
        let cfg = self.get_chain_config_from_step_and_current_session(Step::AaveWithdraw);

        let mut tx = args.clone().partial_transaction;
        tx.input = tx_builders::build_aave_withdraw_tx(args, cfg.aave.clone());
        tx.to = Some(
            Address::from_str(&cfg.aave.lending_pool_address)
                .expect("Invalid lending pool")
                .into_array(),
        );

        self.trigger_signature(Step::AaveWithdraw, tx, callback_gas_tgas)
    }

    pub fn build_and_sign_return_funds_tx(
        &mut self,
        args: RebalancerArgs,
        callback_gas_tgas: u64,
    ) -> Promise {
        self.assert_agent_is_calling();
        let cfg = self.get_chain_config_from_step_and_current_session(Step::RebalancerDeposit);

        let mut tx = args.clone().partial_transaction;
        tx.input = tx_builders::build_return_funds_tx(args);
        tx.to = Some(
            Address::from_str(&cfg.rebalancer.vault_address)
                .expect("Invalid vault")
                .into_array(),
        );

        self.trigger_signature(Step::RebalancerDeposit, tx, callback_gas_tgas)
    }

    pub fn build_and_sign_crosschain_balance_snapshot_tx(
        &self,
        args: SnapshotDigestArgs,
        callback_gas_tgas: u64,
    ) -> Promise {
        self.assert_agent_is_calling();

        let digest = encoders::rebalancer::vault::compute_snapshot_digest(
            args.chain_id,
            args.verifying_contract.clone(),
            args.balance,
            args.nonce,
            args.deadline,
            args.assets,
            args.receiver,
        );

        let payload_hash = digest.try_into().expect("Payload must be 32 bytes long");

        ecdsa::get_sig(payload_hash, PATH.to_string(), KEY_VERSION).then(
            this_contract::ext(env::current_account_id())
                .with_static_gas(Gas::from_tgas(callback_gas_tgas))
                .sign_crosschain_balance_callback(),
        )
    }

    /*
    GLOBAL ALLOWANCE METHODS
    */
    pub fn build_and_sign_approve_vault_to_manage_agents_usdc_tx(
        &mut self,
        partial_transaction: EVMTransaction,
        callback_gas_tgas: u64,
    ) -> Promise {
        self.assert_agent_is_calling();
        let config = self.get_chain_config(&self.source_chain);
        let mut tx = partial_transaction;
        tx.input = tx_builders::build_approve_vault_to_manage_agents_usdc_tx(
            config.rebalancer.vault_address.clone(),
        );
        tx.to = Some(
            Address::from_str(&config.cctp.usdc_address)
                .expect("Invalid USDC address")
                .into_array(),
        );

        self.trigger_signature_without_step_verification(tx, callback_gas_tgas)
    }

    pub(crate) fn trigger_signature(
        &mut self,
        step: Step,
        tx: EVMTransaction,
        callback_gas_tgas: u64,
    ) -> Promise {
        self.assert_step_is_next(step);

        let nonce = self.get_active_session().nonce;
        let payload_hash = self.hash_payload(&tx);
        let key = CacheKey::new(nonce, step as u8);

        if let Some(prev) = self.payload_hashes_by_nonce_and_type.get(&key) {
            if *prev == payload_hash {
                env::panic_str("Signature already cached for this step");
            }
        }

        ecdsa::get_sig(payload_hash, PATH.to_string(), KEY_VERSION).then(
            this_contract::ext(env::current_account_id())
                .with_static_gas(Gas::from_tgas(callback_gas_tgas))
                .sign_callback(nonce, step as u8, tx),
        )
    }

    pub(crate) fn trigger_signature_without_step_verification(
        &mut self,
        tx: EVMTransaction,
        callback_gas_tgas: u64,
    ) -> Promise {
        let payload_hash = self.hash_payload(&tx);

        ecdsa::get_sig(payload_hash, PATH.to_string(), KEY_VERSION).then(
            this_contract::ext(env::current_account_id())
                .with_static_gas(Gas::from_tgas(callback_gas_tgas))
                .sign_generic_callback(tx),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{AaveConfig, CCTPConfig, RebalancerConfig};
    use near_sdk::NearToken;
    use std::str::FromStr;

    use super::*;
    use near_sdk::{test_utils::VMContextBuilder, testing_env, AccountId};

    const ONE_NEAR: NearToken = NearToken::from_near(1);
    const OWNER: &str = "owner.testnet";
    const _WORKER: &str = "worker.testnet";

    fn set_context(predecessor: &str, amount: NearToken) {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor.parse().unwrap());
        builder.attached_deposit(amount);

        testing_env!(builder.build());
    }

    #[test]
    fn test_init() {
        set_context(OWNER, ONE_NEAR);
        let source_chain = ChainId::from_str("1").unwrap();
        let configs = vec![ChainConfig {
            chain_id: ChainId::from_str("2").unwrap(),
            config: Config {
                rebalancer: RebalancerConfig {
                    vault_address: "0xVaultAddress".to_string(),
                },
                cctp: CCTPConfig {
                    messenger_address: "0xMessengerAddress".to_string(),
                    transmitter_address: "0xTransmitterAddress".to_string(),
                    usdc_address: "0xUSDCAddress".to_string(),
                },
                aave: AaveConfig {
                    asset: "0xAaveAssetAddress".to_string(),
                    lending_pool_address: "0xLendingPoolAddress".to_string(),
                    on_behalf_of: "0xOnBehalfOfAddress".to_string(),
                    referral_code: 0,
                },
            },
        }];

        let contract = Contract::init(source_chain, configs);

        assert_eq!(contract.owner_id, AccountId::from_str(OWNER).unwrap());
        assert_eq!(contract.source_chain, source_chain);
        assert_eq!(contract.supported_chains.len(), 1);
        assert_eq!(
            contract.supported_chains[0],
            ChainId::from_str("2").unwrap()
        );
        assert!(contract.active_session.is_none());
        assert!(contract.approved_codehashes.is_empty());
        assert!(contract.worker_by_account_id.is_empty());
        assert!(contract
            .config
            .contains_key(&ChainId::from_str("2").unwrap()));
        assert!(contract.logs.is_empty());
    }
}
