from helpers import broadcast
from ..strategy_context import StrategyContext
from .step import Step

class ApproveVaulToSpendAgentUSDC(Step):
    NAME = "ApproveVaulToSpendAgentUSDC"

    async def run(self, ctx: StrategyContext) -> None:
        spender = ctx.vault_address

        # TODO: Since it is a global approval, we should check if it's already approved to avoid unnecessary transactions
        payload = await ctx.rebalancer_contract.build_and_sign_approve_vault_to_manage_agents_usdc_tx(
            to_chain_id=ctx.to_chain_id,
            to=ctx.usdc_token_address_on_destination_chain,
            spender=spender
        )

        broadcast(ctx.web3_destination, payload)

        print("✅ USDC approved for vault successfully.")