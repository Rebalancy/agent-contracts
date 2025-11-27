from helpers import Assert,broadcaster
from strategy_context import StrategyContext

class WithdrawFromRebalancer:
    NAME = "WithdrawFromRebalancer"

    async def run(self, ctx: StrategyContext) -> None:
        usdc_address = ctx.remote_config[ctx.from_chain_id]["aave"]["asset"]

        payload = await ctx.rebalancer_contract.build_and_sign_withdraw_for_crosschain_allocation_tx(
            source_chain=ctx.from_chain_id,
            amount=ctx.amount,
            to=ctx.vault_address
        )
        
        broadcaster.broadcast(ctx.web3_source, payload)

        Assert.usdc_agent_balance(ctx.web3_source, usdc_address, expected_balance=ctx.amount + ctx.usdc_agent_balance_before_rebalance)