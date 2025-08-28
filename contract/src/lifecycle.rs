use std::str::FromStr;

use near_sdk::{env, Promise};
use omni_transaction::evm::EVMTransaction;

use crate::types::{
    AaveArgs, ActiveSession, ActivityLog, AgentActionType, CacheKey, ChainId, Flow, Step,
};
use crate::{ecdsa, encoders, Contract};

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

        env::log_str(&format!(
            "Rebalance start-phase initiated (nonce {})",
            nonce
        ));
        nonce
    }

    pub fn build_aave_supply_tx(&mut self, args: AaveArgs, callback_gas_tgas: u64) -> Promise {
        self.assert_agent_is_calling();
        self.assert_step_is_next(Step::AaveSupply);
        let cfg = self.get_chain_config_for_step(Step::AaveSupply);

        let input = encoders::aave::lending_pool::encode_supply(
            Address::from_str(&cfg.aave.asset).expect("Invalid asset"),
            U256::from(args.amount),
            Address::from_str(&cfg.aave.on_behalf_of).expect("Invalid on_behalf_of"),
            cfg.aave.referral_code,
        );

        let mut tx = args.partial_transaction;
        tx.input = input;
        tx.to = Some(
            Address::from_str(&cfg.aave.lending_pool_address)
                .expect("Invalid lending pool")
                .into_array(),
        );

        self.trigger_signature(Step::AaveSupply, tx, callback_gas_tgas)
    }

    // 1) AaveWithdraw (SOURCE)
    pub fn build_aave_withdraw_tx(
        &mut self,
        args: AaveArgs, // define AaveWithdrawArgs if different
        callback_gas_tgas: u64,
    ) -> Promise {
        self.assert_agent_is_calling();
        let cfg = self.get_chain_config_for_step(Step::AaveWithdraw);

        // TODO: implement real withdraw encoder (asset, amount, to)
        let input = encoders::aave::lending_pool::encode_withdraw(
            Address::from_str(&cfg.aave.asset).expect("Invalid asset"),
            U256::from(args.amount),
            Address::from_str(&cfg.aave.on_behalf_of).expect("Invalid recipient"),
        );

        let mut tx = args.partial_transaction;
        tx.input = input;
        tx.to = Some(
            Address::from_str(&cfg.aave.lending_pool_address)
                .expect("Invalid lending pool")
                .into_array(),
        );

        self.trigger_signature(Step::AaveWithdraw, tx, callback_gas_tgas)
    }

    // 2) CCTPBurn (SOURCE)
    pub fn build_cctp_burn_tx(&mut self, args: CCTPArgs, callback_gas_tgas: u64) -> Promise {
        self.assert_agent_is_calling();
        let cfg = self.get_chain_config_for_step(Step::CCTPBurn);

        let input = encoders::cctp::messenger::encode_deposit_for_burn(
            U256::from(args.amount),
            args.destination_domain,
            B256::from_str(&args.mint_recipient).expect("Invalid recipient"),
            Address::from_str(&args.burn_token).expect("Invalid token"),
            B256::from_str(&args.destination_caller).expect("Invalid caller"),
            U256::from(args.max_fee),
            args.min_finality_threshold,
        );

        let mut tx = args.partial_burn_transaction;
        tx.input = input;
        tx.to = Some(
            Address::from_str(&cfg.cctp.messenger_address)
                .expect("Invalid messenger")
                .into_array(),
        );

        self.trigger_signature(Step::CCTPBurn, tx, callback_gas_tgas)
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
}
