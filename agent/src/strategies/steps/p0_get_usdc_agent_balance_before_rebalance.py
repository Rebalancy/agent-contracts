from helpers import BalanceHelper
from ..strategy_context import StrategyContext
from .step import Step


class GetUSDCBalanceBeforeRebalance(Step):
    NAME = "GetUSDCBalanceBeforeRebalance"

    async def run(self, ctx: StrategyContext) -> None:
        print("Getting USDC agent balance before rebalance...")
        
        ctx.usdc_agent_balance_before_rebalance = BalanceHelper.get_usdc_agent_balance(ctx.web3_source, ctx.usdc_token_address_on_source_chain)

        if not ctx.usdc_agent_balance_before_rebalance:
            raise ValueError("USDC agent balance before rebalance is not set in context.")
        