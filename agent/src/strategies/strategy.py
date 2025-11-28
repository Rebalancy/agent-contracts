from typing import Dict, Optional
from near_omni_client.providers.evm.alchemy_provider import AlchemyFactoryProvider

from adapters import RebalancerContract
from config import Config

from .strategy_context import StrategyContext
from .steps import retry_async_step

class Strategy:
    NAME = "BaseStrategy"
    STEPS: list[type] = []

    def __init__(self, *, rebalancer_contract: RebalancerContract, evm_factory_provider: AlchemyFactoryProvider, vault_address: str, config: Config, remote_config: Dict[str, dict], agent_address: str):
        self.rebalancer_contract = rebalancer_contract
        self.evm_factory_provider = evm_factory_provider
        self.vault_address = vault_address
        self.config = config
        self.remote_config = remote_config
        self.agent_address = agent_address

    async def execute(self, *, from_chain_id: int, to_chain_id: int, amount: int, restart_from: str | None = None):
        ctx = self._make_context(
            from_chain_id=from_chain_id,
            to_chain_id=to_chain_id,
            amount=amount
        )
        await self._run_phases(ctx, restart_from)

    def _make_context(self, *, from_chain_id: int, to_chain_id: int, amount: int) -> StrategyContext:
        print(f"🟩 Flow {self.NAME} | from={from_chain_id} to={to_chain_id} amount={amount}")
        return StrategyContext(
            from_chain_id=from_chain_id,
            to_chain_id=to_chain_id,
            amount=amount,
            remote_config=self.remote_config,
            config=self.config,
            agent_address=self.agent_address,
            vault_address=self.vault_address,
            evm_factory_provider=self.evm_factory_provider,
            rebalancer_contract=self.rebalancer_contract
        )

    async def _run_phases(self, ctx: StrategyContext, restart_from: Optional[str] = None):
        start_index = 0

        if restart_from:
            for i, step_cls in enumerate(self.STEPS):
                if step_cls.NAME == restart_from:
                    start_index = i
                    break

        for step_cls in self.STEPS[start_index:]:
            step = step_cls()
            print(f"➡️ Phase: {step.NAME}")
            await retry_async_step(lambda: step.run(ctx))