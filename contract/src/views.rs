use alloy_primitives::{Address, B256, U256};
use near_sdk::{near, AccountId};
use std::str::FromStr;

use crate::{
    encoders,
    types::{ActiveSession, ActivityLog, CacheKey, ChainId, Config, Flow, Step, Worker},
    Contract, ContractExt,
};

#[near]
impl Contract {
    pub fn get_source_chain(&self) -> ChainId {
        self.source_chain.clone()
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

    pub fn get_allocations(&self) -> Vec<(ChainId, u128)> {
        self.supported_chains
            .iter()
            .map(|chain_id| {
                let allocation = self.allocations.get(chain_id).cloned().unwrap_or(0);
                (chain_id.clone(), allocation)
            })
            .collect()
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

    pub fn get_chain_config(&self, destination_chain: &ChainId) -> &Config {
        self.config
            .get(destination_chain)
            .expect("Chain not configured")
    }

    pub fn get_active_session(&self) -> &ActiveSession {
        self.active_session.as_ref().expect("No active session")
    }

    pub fn get_signed_transactions(&self, nonce: u64) -> Vec<Vec<u8>> {
        self.logs
            .get(&nonce)
            .map(|log| log.transactions.clone())
            .unwrap_or_default()
    }

    pub fn get_signature(&self, nonce: u64, tx_type: u8) -> Option<Vec<u8>> {
        let cache_key = CacheKey { nonce, tx_type };
        self.signatures_by_nonce_and_type.get(&cache_key).cloned()
    }

    pub fn get_activity_log(&self) -> ActivityLog {
        let nonce = self.get_active_session().nonce;
        self.logs.get(&nonce).expect("Log not found").clone()
    }

    // Transaction Input Builders
    pub fn build_cctp_burn_tx(
        &self,
        amount: u128,
        destination_domain: u32,
        mint_recipient: String,
        burn_token: String,
        destination_caller: String,
        max_fee: u128,
        min_finality_threshold: u32,
    ) -> Vec<u8> {
        encoders::cctp::messenger::encode_deposit_for_burn(
            U256::from(amount),
            destination_domain,
            B256::from_str(&mint_recipient).expect("Invalid recipient"),
            Address::from_str(&burn_token).expect("Invalid token address"),
            B256::from_str(&destination_caller).expect("Invalid destination caller"),
            U256::from(max_fee),
            min_finality_threshold,
        )
    }

    pub fn build_cctp_mint_tx(&self, message: Vec<u8>, attestation: Vec<u8>) -> Vec<u8> {
        encoders::cctp::transmitter::encode_receive_message(message, attestation)
    }

    pub fn build_aave_supply_tx(
        &self,
        asset: String,
        amount: u128,
        on_behalf_of: String,
        referral_code: u16,
    ) -> Vec<u8> {
        encoders::aave::lending_pool::encode_supply(
            Address::from_str(&asset).expect("Invalid asset address"),
            U256::from(amount),
            Address::from_str(&on_behalf_of).expect("Invalid on_behalf_of address"),
            referral_code,
        )
    }

    pub fn build_aave_withdraw_tx(
        &self,
        asset: String,
        amount: u128,
        on_behalf_of: String,
    ) -> Vec<u8> {
        encoders::aave::lending_pool::encode_withdraw(
            Address::from_str(&asset).expect("Invalid asset address"),
            U256::from(amount),
            Address::from_str(&on_behalf_of).expect("Invalid on_behalf_of address"),
        )
    }

    pub fn build_withdraw_for_crosschain_allocation_tx(
        &self,
        amount: u128,
        cross_chain_a_token_balance: Option<u128>,
    ) -> Vec<u8> {
        encoders::rebalancer::vault::encode_withdraw_for_crosschain_allocation(
            amount,
            cross_chain_a_token_balance.unwrap_or(0),
        )
    }

    pub fn build_return_funds_tx(
        &self,
        amount: u128,
        cross_chain_a_token_balance: Option<u128>,
    ) -> Vec<u8> {
        encoders::rebalancer::vault::encode_return_funds(
            amount,
            cross_chain_a_token_balance.unwrap_or(0),
        )
    }

    pub fn get_pending_step(&self) -> Option<Step> {
        if let Some(session) = &self.active_session {
            for &st in session.flow.sequence() {
                if !self.has_signature(st) {
                    return Some(st);
                }
            }
            None // it means all steps are signed
        } else {
            None
        }
    }

    pub fn get_active_session_info(&self) -> Option<(u64, Flow, Option<Step>)> {
        if let Some(session) = &self.active_session {
            let mut pending = None;
            for &st in session.flow.sequence() {
                if !self.has_signature(st) {
                    pending = Some(st);
                    break;
                }
            }
            Some((session.nonce, session.flow.clone(), pending))
        } else {
            None
        }
    }
}
