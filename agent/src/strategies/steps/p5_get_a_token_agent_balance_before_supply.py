from adapters import LendingPool
from helpers import BalanceHelper
from ..strategy_context import StrategyContext
from .step import Step


class GetATokenBalanceBeforeSupply(Step):
    NAME = "GetATokenBalanceBeforeSupply"

    async def run(self, ctx: StrategyContext) -> None:
        print("Getting AToken balance before supply...")

        ctx.a_token_address_on_destination_chain = LendingPool.get_atoken_address(ctx.web3_destination, ctx.aave_lending_pool_address_on_destination_chain, ctx.usdc_token_address_on_destination_chain)

        if not ctx.a_token_address_on_destination_chain:
            raise ValueError("Failed to get AToken address on destination chain.")
        
        ctx.a_token_balance_before_supply = BalanceHelper.get_atoken_agent_balance(ctx.web3_destination, ctx.a_token_address_on_destination_chain)

        print(f"AToken balance before supply: {ctx.a_token_balance_before_supply}")

        print("✅ AToken balance before supply retrieved successfully.")
