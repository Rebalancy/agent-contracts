from typing import Dict
from near_omni_client.providers.evm import AlchemyFactoryProvider

from .strategy import Strategy
from config import Config
from adapters import RebalancerContract

# TODO: Fix this
class AaveToAave(Strategy):
    def __init__(self, *, rebalancer_contract: RebalancerContract, evm_factory_provider: AlchemyFactoryProvider, vault_address: str, config: Config, remote_config: Dict[str, dict], agent_address: str) -> None:
        self.rebalancer_contract = rebalancer_contract
        self.evm_factory_provider = evm_factory_provider
        self.vault_address = vault_address
        self.config = config
        self.remote_config = remote_config
        self.agent_address = agent_address

    async def execute(self, *, from_chain_id: int, to_chain_id: int, amount: int) -> None:
        print(f"🟦 Flow Rebalancer→Aave | from={from_chain_id} to={to_chain_id} amount={amount}")
        pass

