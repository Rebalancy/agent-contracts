from helpers import broadcast
from ..strategy_context import StrategyContext
from .step import Step

class ApproveAaveUSDCBeforeSupply(Step):
    NAME = "ApproveAaveUSDCBeforeSupply"

    async def run(self, ctx: StrategyContext) -> None:
        spender = ctx.aave_lending_pool_address_on_destination_chain

        payload = await ctx.rebalancer_contract.build_and_sign_aave_approve_before_supply_tx(
            to_chain_id=ctx.to_chain_id,
            amount=int(ctx.amount * 2),
            spender=spender,
            to=ctx.usdc_token_address_on_destination_chain
        )

        broadcast(ctx.web3_destination, payload)

        print("✅ USDC approved for Aave supply successfully.")