from helpers import broadcast
from ..strategy_context import StrategyContext
from .step import Step

class WithdrawFromAave(Step):
    NAME = "WithdrawFromAave"

    async def run(self, ctx: StrategyContext) -> None:
        asset = ctx.usdc_token_address_on_source_chain
        on_behalf = ctx.agent_address
        # @dev since we can only interact directly with an Aave Lending Pool contract in a NON-Source chain,
        # it means that we are trying to move funds from the NON-Source chain, therefore the chain id is the from_chain_id
        payload = await ctx.rebalancer_contract.build_and_sign_aave_withdraw_tx(
            chain_id=ctx.from_chain_id,
            asset=asset,
            amount=ctx.amount,
            on_behalf_of=on_behalf,
            to=ctx.aave_lending_pool_address_on_source_chain
        )
        broadcast(ctx.web3_source, payload)

        print("✅ Withdraw from Aave transaction broadcasted successfully!")