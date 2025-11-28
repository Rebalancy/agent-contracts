from helpers import broadcast
from ..strategy_context import StrategyContext
from .step import Step


class WithdrawFromRebalancer(Step):
    NAME = "WithdrawFromRebalancer"

    async def run(self, ctx: StrategyContext) -> None:
        payload = await ctx.rebalancer_contract.build_and_sign_withdraw_for_crosschain_allocation_tx(
            source_chain=ctx.from_chain_id,
            amount=ctx.amount,
            to=ctx.vault_address
        )
        
        broadcast(ctx.web3_source, payload)

        print(f"Withdrew {ctx.amount} USDC from rebalancer on chain {ctx.from_chain_id}.")