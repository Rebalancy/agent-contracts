from typing import Dict
from tx_types import Flow
from strategies import Strategy, AaveToAave, RebalancerToAave, AaveToRebalancer
from rebalancer_contract import RebalancerContract

class StrategyManager:
    _strategies: Dict[Flow, Strategy] | None = None

    @classmethod
    def configure(cls, *, rebalancer_contract: RebalancerContract, evm_factory_provider, vault_address: str) -> None:
        cls._strategies = {
            Flow.RebalancerToAave: RebalancerToAave(rebalancer_contract=rebalancer_contract, evm_factory_provider=evm_factory_provider, vault_address=vault_address),
            Flow.AaveToRebalancer: AaveToRebalancer(rebalancer_contract=rebalancer_contract, evm_factory_provider=evm_factory_provider, vault_address=vault_address),
            Flow.AaveToAave:       AaveToAave(rebalancer_contract=rebalancer_contract, evm_factory_provider=evm_factory_provider, vault_address=vault_address),
        }

    @classmethod
    def get_strategy(cls, flow: Flow) -> Strategy:
        if cls._strategies is None:
            raise RuntimeError("Strategy not configured. Call 'configure' first.")
        try:
            return cls._strategies[flow]
        except KeyError as e:
            raise KeyError(f"No strategy found for {flow}") from e