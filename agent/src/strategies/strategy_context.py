from config import Config
from adapters import RebalancerContract
from utils import from_chain_id_to_network

from near_omni_client.providers.evm import AlchemyFactoryProvider
from near_omni_client.networks import Network
from near_omni_client.adapters.cctp.attestation_service_types import Message
from web3 import Web3

class StrategyContext:
    def __init__(
        self,
        *,
        from_chain_id: int,
        to_chain_id: int,
        amount: int,
        remote_config: dict,
        config: Config,
        agent_address: str,
        vault_address: str,
        evm_factory_provider: AlchemyFactoryProvider,
        rebalancer_contract: RebalancerContract,
    ):
        self.from_chain_id = from_chain_id
        self.to_chain_id = to_chain_id
        self.amount = amount

        self.remote_config = remote_config
        self.config = config
        self.agent_address = agent_address
        self.vault_address = vault_address

        self.evm_factory_provider = evm_factory_provider
        self.rebalancer_contract = rebalancer_contract
        
        self.from_network_id = from_chain_id_to_network(from_chain_id)
        self.to_network_id = from_chain_id_to_network(to_chain_id)
        self.web3_source = evm_factory_provider.get_provider(self.from_network_id)
        self.web3_destination = evm_factory_provider.get_provider(self.to_network_id)

        self.usdc_token_address_on_source_chain = str | None
        self.usdc_token_address_on_destination_chain = str | None

        # ===== filled by phases =====
        self.nonce = int | None
        self.cctp_fees = int | None
        self.burn_tx_hash = str | None
        self.attestation = Message | None
    
        # self.usdc_agent_balance_before = int | None
        # self.a_token_address = str | None
        # self.aave_lending_pool_address = None