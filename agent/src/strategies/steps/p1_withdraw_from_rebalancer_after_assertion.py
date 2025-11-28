from helpers import Assert,broadcaster
from ..strategy_context import StrategyContext
from .step import Step


class WithdrawFromRebalancerAfterAssertion(Step):
    NAME = "WithdrawFromRebalancerAfterAssertion"

    async def run(self, ctx: StrategyContext) -> None:
        Assert.usdc_agent_balance(ctx.web3_source, ctx.usdc_token_address_on_source_chain, expected_balance=ctx.amount + ctx.usdc_agent_balance_before_rebalance)