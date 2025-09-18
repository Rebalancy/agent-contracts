use near_sdk::{near, AccountId};

use crate::{
    tx_builders,
    types::{
        AaveArgs, ActiveSession, ActivityLog, CCTPBurnArgs, CCTPMintArgs, CacheKey, ChainId,
        Config, RebalancerArgs, Worker,
    },
    {Contract, ContractExt},
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
    pub fn build_cctp_burn_tx(&self, args: CCTPBurnArgs) -> Vec<u8> {
        tx_builders::build_cctp_burn_tx(args)
    }

    pub fn build_cctp_mint_tx(&self, args: CCTPMintArgs) -> Vec<u8> {
        tx_builders::build_cctp_mint_tx(args)
    }

    pub fn build_aave_supply_tx(&self, args: AaveArgs, config: Config) -> Vec<u8> {
        tx_builders::build_aave_supply_tx(args, config.aave.clone())
    }

    pub fn build_aave_withdraw_tx(&self, args: AaveArgs, config: Config) -> Vec<u8> {
        tx_builders::build_aave_withdraw_tx(args, config.aave.clone())
    }

    pub fn build_withdraw_for_crosschain_allocation_tx(&self, args: RebalancerArgs) -> Vec<u8> {
        tx_builders::build_withdraw_for_crosschain_allocation_tx(args)
    }

    pub fn build_return_funds_tx(&self, args: RebalancerArgs) -> Vec<u8> {
        tx_builders::build_return_funds_tx(args)
    }
}
