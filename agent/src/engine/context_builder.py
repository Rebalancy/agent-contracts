from dataclasses import dataclass

from near_omni_client.providers.near import NearFactoryProvider
from near_omni_client.providers.evm import AlchemyFactoryProvider
from near_omni_client.wallets.near_wallet import NearWallet
from near_omni_client.wallets import MPCWallet
from near_omni_client.crypto.keypair import KeyPair
from near_omni_client.chain_signatures.kdf import Kdf
from near_omni_client.chain_signatures.utils import get_evm_address
from near_omni_client.json_rpc.client import NearClient
from near_omni_client.networks import Network

from utils import from_chain_id_to_network
from helpers import GasEstimator
from adapters import RebalancerContract
from config import Config


@dataclass
class EngineContext:
    near_client: NearClient
    evm_factory_provider: AlchemyFactoryProvider
    near_wallet: NearWallet
    mpc_wallet: MPCWallet
    rebalancer_contract: RebalancerContract
    gas_estimator: GasEstimator
    agent_address: str
    source_chain_id: int
    source_network: Network
    remote_configs: dict
    vault_address: str


async def build_context(config: Config) -> EngineContext:
    # ---------------------------
    # NEAR provider & client
    # ---------------------------
    near_factory = NearFactoryProvider()
    near_client = near_factory.get_provider(config.near_network)

    # ---------------------------
    # EVM provider factory
    # ---------------------------
    alchemy_factory_provider = AlchemyFactoryProvider(api_key=config.alchemy_api_key)

    # ---------------------------
    # Agent KDF → EVM address
    # ---------------------------
    root_pubkey = Kdf.get_root_public_key(config.network_short_name)
    epsilon = Kdf.derive_epsilon(account_id=config.contract_id, path=config.kdf_path)
    agent_public_key = Kdf.derive_public_key(
        root_public_key_str=root_pubkey,
        epsilon=epsilon,
    )
    agent_address = get_evm_address(agent_public_key)

    # ---------------------------
    # Gas estimator
    # ---------------------------
    gas_estimator = GasEstimator(evm_factory_provider=alchemy_factory_provider)

    # ---------------------------
    # One-time NEAR signer
    # ---------------------------
    near_local_signer = KeyPair.from_string(config.one_time_signer_private_key)
    near_wallet = NearWallet(
        keypair=near_local_signer,
        account_id=config.one_time_signer_account_id,
        provider_factory=near_factory,
        supported_networks=config.supported_near_networks,
    )

    # ---------------------------
    # MPC Wallet for EVM signing
    # ---------------------------
    mpc_wallet = MPCWallet(
        path=config.kdf_path,
        account_id=config.contract_id,
        near_network=config.near_network,
        provider_factory=alchemy_factory_provider,
        supported_networks=config.supported_evm_networks,
    )

    # ---------------------------
    # Contract wrapper
    # ---------------------------
    rebalancer_contract = RebalancerContract(
        near_client=near_client,
        near_wallet=near_wallet,
        near_contract_id=config.contract_id,
        agent_address=agent_address,
        gas_estimator=gas_estimator,
        evm_provider=alchemy_factory_provider,
        config=config,
    )

    # ---------------------------
    # Pull remote configs once
    # ---------------------------
    remote_configs = await rebalancer_contract.get_all_configs()

    # ---------------------------
    # Pull source chain ID
    # ---------------------------
    source_chain_id = await rebalancer_contract.get_source_chain()
    source_network = from_chain_id_to_network(source_chain_id)
    
    # ---------------------------
    # Source Chain Config 
    # ---------------------------
    source_chain_config = remote_configs.get(source_chain_id, None)
    if not source_chain_config:
        raise ValueError(f"Source chain config for chain ID {source_chain_id} not found in remote configs.")
    
    # ---------------------------
    # Vault Address
    # ---------------------------
    vault_address = source_chain_config["rebalancer"]["vault_address"]
    if not vault_address:
        raise ValueError(f"Vault address for source chain {source_chain_id} not found in remote configs.")

    # ---------------------------
    # Build context object
    # ---------------------------
    return EngineContext(
        near_client=near_client,
        evm_factory_provider=alchemy_factory_provider,
        near_wallet=near_wallet,
        mpc_wallet=mpc_wallet,
        rebalancer_contract=rebalancer_contract,
        gas_estimator=gas_estimator,
        agent_address=agent_address,
        remote_configs=remote_configs,
        source_chain_id=source_chain_id,
        source_network=source_network,
        vault_address=vault_address,
    )