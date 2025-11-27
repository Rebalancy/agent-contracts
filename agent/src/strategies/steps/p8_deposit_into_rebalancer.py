from helpers import broadcaster
from ..strategy_context import StrategyContext

class DepositIntoRebalancer:
    NAME = "DepositIntoRebalancer"

    async def run(self, ctx: StrategyContext):
        payload = await ctx.rebalancer_contract.build_rebalancer_deposit_tx(
            to_chain_id=ctx.to_chain_id,
            amount=ctx.amount
        )
        broadcaster.broadcast(ctx.web3_destination, payload)