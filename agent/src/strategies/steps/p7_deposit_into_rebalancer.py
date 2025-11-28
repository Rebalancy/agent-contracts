from helpers import broadcast
from ..strategy_context import StrategyContext

class DepositIntoRebalancer:
    NAME = "DepositIntoRebalancer"

    async def run(self, ctx: StrategyContext):
        payload = await ctx.rebalancer_contract.build_rebalancer_deposit_tx(
            to_chain_id=ctx.to_chain_id,
            amount=ctx.amount
        )
        broadcast(ctx.web3_destination, payload)