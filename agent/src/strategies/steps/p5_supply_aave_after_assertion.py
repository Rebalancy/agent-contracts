import time
from helpers import Assert
from ..strategy_context import StrategyContext
from .step import Step

class SupplyAaveAfterAssertion(Step):
    NAME = "SupplyAaveAfterAssertion"

    async def run(self, ctx: StrategyContext) -> None:
        Assert.atoken_agent_balance(
            ctx.web3_destination, 
            ctx.a_token_address_on_destination_chain,
            ctx.a_token_balance_before_supply + ctx.amount
        )

        print("✅ AToken balance assertion after Aave supply passed successfully.")