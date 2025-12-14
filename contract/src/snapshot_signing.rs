use crate::{
    constants::*, ecdsa, encoders, external::this_contract, types::SnapshotDigestArgs, Contract,
    ContractExt,
};
use near_sdk::{env, near, Gas, Promise};

#[near]
impl Contract {
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
}

#[cfg(test)]
mod maintests {}

// TODO
// pub owner_id: AccountId, -> No cambia
// pub source_chain: ChainId, -> No cambia
// pub approved_codehashes: IterableSet<String>, -> Si cambia
// pub worker_by_account_id: IterableMap<AccountId, Worker>, -> Si cambia
// pub config: LookupMap<ChainId, Config>, -> No cambia
// pub logs: IterableMap<u64, ActivityLog>, -> Si cambia
// pub logs_nonce: u64, -> Si cambia
// pub supported_chains: Vec<ChainId>, -> No cambia
// pub active_session: Option<ActiveSession>, -> Si cambia

// TODO: Assert estos 2
// pub signatures_by_nonce_and_type: LookupMap<CacheKey, Vec<u8>>, // Si cambia
// pub payload_hashes_by_nonce_and_type: LookupMap<CacheKey, [u8; 32]>, // Si cambia
// TODO: Improvements
// Support add new configurations and new chains
// Withdrawals
// Emergency stop
