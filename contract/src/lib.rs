use crate::{
    constants::{KEY_VERSION, PATH},
    external::this_contract,
    types::{ActiveSession, ActivityLog, CacheKey, ChainConfig, ChainId, Config, Step, Worker},
};
use near_sdk::{
    env, near,
    store::{IterableMap, IterableSet, LookupMap},
    AccountId, Gas, PanicOnDefault, Promise,
};
use omni_transaction::evm::EVMTransaction;

mod access_control;
mod admin;
mod agent;
mod allowances;
mod callbacks;
mod collateral;
mod constants;
mod ecdsa;
mod encoders;
mod external;
mod snapshot_signing;
mod state_machine;
mod steps;
mod tx_builders;
pub mod types;
mod views;

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

    // TODO: Integrate this
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
                // TODO: Fix this to return the existing signature
                // let hashed_signature = self
                //     .signatures_by_nonce_and_type
                //     .get(&key)
                //     .expect("Signature must be present if payload hash matches");

                // return PromiseOrValue::Value(hashed_signature.clone());
            }
        }

        ecdsa::get_sig(payload_hash, PATH.to_string(), KEY_VERSION).then(
            this_contract::ext(env::current_account_id())
                .with_static_gas(Gas::from_tgas(callback_gas_tgas))
                .sign_callback(nonce, step as u8, tx),
        )
    }
}

#[cfg(test)]
mod test_helpers {
    use super::types::*;
    use super::Contract;
    use near_sdk::NearToken;
    use near_sdk::{test_utils::VMContextBuilder, testing_env};
    use omni_transaction::signer::types::{
        SerializableAffinePoint, SerializableScalar, SignatureResponse,
    };
    use std::str::FromStr;

    pub const ONE_NEAR: NearToken = NearToken::from_near(1);
    pub const OWNER: &str = "owner.testnet";
    pub const _WORKER: &str = "worker.testnet";
    pub const DEFAULT_ATTACHED_DEPOSIT: NearToken = ONE_NEAR;

    pub fn set_context(predecessor: &str) {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor.parse().unwrap());
        builder.attached_deposit(DEFAULT_ATTACHED_DEPOSIT);

        testing_env!(builder.build());
    }

    pub fn set_context_with_attached_deposit(predecessor: &str, amount: NearToken) {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor.parse().unwrap());
        builder.attached_deposit(amount);

        testing_env!(builder.build());
    }

    pub const DEFAULT_SOURCE_CHAIN_STR: &str = "1";
    pub const DEFAULT_DESTINATION_CHAIN_STR: &str = "2";
    pub const DEFAULT_SOURCE_CHAIN: ChainId = 1;
    pub const DEFAULT_DESTINATION_CHAIN: ChainId = 2;

    fn init_contract_with(source_chain: ChainId, configs: Vec<ChainConfig>) -> Contract {
        Contract::init(source_chain, configs)
    }

    pub fn init_contract_with_defaults() -> Contract {
        let source_chain = DEFAULT_SOURCE_CHAIN;
        let configs = build_fake_configs();

        init_contract_with(source_chain, configs)
    }

    // Utilities

    impl Contract {
        fn assert_state_is(&self, expected: &Contract) {
            assert!(self.owner_id == expected.owner_id);
            assert!(self.source_chain == expected.source_chain);
            assert!(self.approved_codehashes.len() == expected.approved_codehashes.len());
            assert!(self.worker_by_account_id.len() == expected.worker_by_account_id.len());
            assert!(self.logs.len() == expected.logs.len());
        }
    }
    // fn assert_state_is(contract: &Contract, expected: &Contract) {
    //     assert!(contract.owner_id == expected.owner_id);
    //     assert!(contract.source_chain == expected.source_chain);
    //     assert!(contract.approved_codehashes.len() == expected.approved_codehashes.len());
    //     assert!(contract.worker_by_account_id.len() == expected.worker_by_account_id.len());
    //     assert!(contract.logs.len() == expected.logs.len());
    // }

    fn build_fake_configs() -> Vec<ChainConfig> {
        vec![
            ChainConfig {
                chain_id: DEFAULT_SOURCE_CHAIN,
                config: Config {
                    rebalancer: RebalancerConfig {
                        vault_address: "0xE168d95f8d1B8EC167A63c8E696076EC8EE95337".to_string(),
                    },
                    cctp: CCTPConfig {
                        messenger_address: "0x8FE6B999Dc680CcFDD5Bf7EB0974218be2542DAA".to_string(),
                        transmitter_address: "0xe737e5cebeeba77efe34d4aa090756590b1ce275"
                            .to_string(),
                        usdc_address: "0x75faf114eafb1BDbe2F0316DF893fd58CE46AA4d".to_string(),
                    },
                    aave: AaveConfig {
                        asset: "0x75faf114eafb1BDbe2F0316DF893fd58CE46AA4d".to_string(),
                        lending_pool_address: "0xBfC91D59fdAA134A4ED45f7B584cAf96D7792Eff"
                            .to_string(),
                        on_behalf_of: "0x20f2747bbc52453ac0774b5b2fe0e28dc6637f30".to_string(),
                        referral_code: 0,
                    },
                },
            },
            ChainConfig {
                chain_id: DEFAULT_DESTINATION_CHAIN,
                config: Config {
                    rebalancer: RebalancerConfig {
                        vault_address: "".into(),
                    },
                    cctp: CCTPConfig {
                        messenger_address: "0x8FE6B999Dc680CcFDD5Bf7EB0974218be2542DAA".into(),
                        transmitter_address: "0xe737e5cebeeba77efe34d4aa090756590b1ce275".into(),
                        usdc_address: "0x75faf114eafb1BDbe2F0316DF893fd58CE46AA4d".into(),
                    },
                    aave: AaveConfig {
                        asset: "0x75faf114eafb1BDbe2F0316DF893fd58CE46AA4d".into(),
                        lending_pool_address: "0xBfC91D59fdAA134A4ED45f7B584cAf96D7792Eff".into(),
                        on_behalf_of: "0x20f2747bbc52453ac0774b5b2fe0e28dc6637f30".into(),
                        referral_code: 0,
                    },
                },
            },
        ]
    }

    fn build_mock_signature() -> SignatureResponse {
        SignatureResponse {
            big_r: SerializableAffinePoint {
                affine_point: hex::encode(vec![0u8; 33]),
            },
            s: SerializableScalar {
                scalar: hex::encode(vec![1u8; 32]),
            },
            recovery_id: 1,
        }
    }

    impl ChainConfig {
        pub fn with_chain_id(mut self, id: &str) -> Self {
            self.chain_id = ChainId::from_str(id).unwrap();
            self
        }

        pub fn with_config(mut self, config: Config) -> Self {
            self.config = config;
            self
        }
    }

    impl Config {
        pub fn with_aave_config(mut self, aave: AaveConfig) -> Self {
            self.aave = aave;
            self
        }

        pub fn with_cctp_config(mut self, cctp: CCTPConfig) -> Self {
            self.cctp = cctp;
            self
        }

        pub fn with_rebalancer_config(mut self, rebalancer: RebalancerConfig) -> Self {
            self.rebalancer = rebalancer;
            self
        }
    }

    impl AaveConfig {
        pub fn with_asset(mut self, asset: &str) -> Self {
            self.asset = asset.to_string();
            self
        }

        pub fn with_lending_pool_address(mut self, address: &str) -> Self {
            self.lending_pool_address = address.to_string();
            self
        }

        pub fn with_on_behalf_of(mut self, on_behalf_of: &str) -> Self {
            self.on_behalf_of = on_behalf_of.to_string();
            self
        }

        pub fn with_referral_code(mut self, code: u16) -> Self {
            self.referral_code = code;
            self
        }
    }

    impl RebalancerConfig {
        pub fn with_vault_address(mut self, address: &str) -> Self {
            self.vault_address = address.to_string();
            self
        }
    }

    impl CCTPConfig {
        pub fn with_messenger_address(mut self, address: &str) -> Self {
            self.messenger_address = address.to_string();
            self
        }

        pub fn with_transmitter_address(mut self, address: &str) -> Self {
            self.transmitter_address = address.to_string();
            self
        }

        pub fn with_usdc_address(mut self, address: &str) -> Self {
            self.usdc_address = address.to_string();
            self
        }
    }
}

#[cfg(test)]
mod maintests {
    use crate::test_helpers::*;
    use near_sdk::AccountId;

    use std::str::FromStr;

    #[test]
    fn test_init() {
        set_context(OWNER);

        let contract = init_contract_with_defaults();

        assert_eq!(contract.owner_id, AccountId::from_str(OWNER).unwrap());
        assert_eq!(contract.source_chain, DEFAULT_SOURCE_CHAIN);
        assert!(contract.approved_codehashes.is_empty()); // @dev since the agent registers itself later on
        assert!(contract.worker_by_account_id.is_empty()); // @dev since the agent registers itself later on
        assert!(contract.logs.is_empty());
        assert_eq!(contract.logs_nonce, 0);
        assert_eq!(contract.supported_chains.len(), 2);
        assert_eq!(contract.supported_chains[0], DEFAULT_SOURCE_CHAIN);
        assert_eq!(contract.supported_chains[1], DEFAULT_DESTINATION_CHAIN);
        assert!(contract.active_session.is_none());
        assert!(contract.config.contains_key(&DEFAULT_SOURCE_CHAIN));
        assert!(contract.config.contains_key(&DEFAULT_DESTINATION_CHAIN));
    }
}
