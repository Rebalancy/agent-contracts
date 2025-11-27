from helpers import BalanceHelper
from engine_types import Flow
from ..strategy_context import StrategyContext

class StartRebalance:
    NAME = "StartRebalance"

    async def run(self, ctx: StrategyContext) -> None:
        ctx.nonce = await ctx.rebalancer_contract.start_rebalance(
            flow=Flow.RebalancerToAave,
            source_chain=ctx.from_chain_id,
            destination_chain=ctx.to_chain_id,
            expected_amount=ctx.amount
        )
        print(f"Started rebalance with nonce: {ctx.nonce}")
        ctx.usdc_agent_balance_before_rebalance = BalanceHelper.get_usdc_agent_balance(ctx.web3_source, ctx.usdc_token_address_on_source_chain)