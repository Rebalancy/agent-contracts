from helpers import BalanceHelper
from engine_types import Flow
from ..strategy_context import StrategyContext
from .step import Step


class StartRebalance(Step):
    NAME = "StartRebalance"

    async def run(self, ctx: StrategyContext) -> None:
        print("Starting rebalance...")
        ctx.nonce = await ctx.rebalancer_contract.start_rebalance(
            flow=Flow.RebalancerToAave,
            source_chain=ctx.from_chain_id,
            destination_chain=ctx.to_chain_id,
            expected_amount=ctx.amount
        )
        print(f"Started rebalance with nonce: {ctx.nonce}")
