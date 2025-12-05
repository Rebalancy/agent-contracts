from ..strategy_context import StrategyContext
from .step import Step


class CompleteRebalance(Step):
    NAME = "CompleteRebalance"

    async def run(self, ctx: StrategyContext) -> None:
        print("Completing rebalance...")
        
        ctx.nonce = await ctx.rebalancer_contract.complete_rebalance()
        
        print(f"Completed rebalance with nonce: {ctx.nonce}")
