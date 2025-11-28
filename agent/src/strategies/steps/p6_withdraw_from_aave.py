from helpers import broadcast
from ..strategy_context import StrategyContext
from .step import Step

class WithdrawFromAave(Step):
    NAME = "WithdrawFromAave"

    async def run(self, ctx: StrategyContext) -> None:
        payload = await ctx.rebalancer_contract.build_aave_withdraw_tx(
            from_chain_id=ctx.from_chain_id,
            amount=ctx.amount
        )
        broadcast(ctx.web3_source, payload)