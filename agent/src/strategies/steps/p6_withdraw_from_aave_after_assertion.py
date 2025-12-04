from helpers import Assert
from ..strategy_context import StrategyContext
from .step import Step

class WithdrawFromAaveAfterAssertion(Step):
    NAME = "WithdrawFromAaveAfterAssertion"

    async def run(self, ctx: StrategyContext) -> None:
        # Check USDC balance after withdrawing from Aave + balance before rebalance
        Assert.usdc_agent_balance(ctx.web3_source, ctx.usdc_token_address_on_source_chain, expected_balance=ctx.amount + ctx.usdc_agent_balance_before_rebalance)
        
        print("Assertion after withdrawing from Aave completed successfully.")


