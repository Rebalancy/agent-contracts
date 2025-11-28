from helpers import broadcast
from ..strategy_context import StrategyContext
from .step import Step

class ApproveBeforeCctpBurn(Step):
    NAME = "ApproveBeforeCctpBurn"

    async def run(self, ctx: StrategyContext):
        spender = ctx.messenger_address_on_source_chain # the messenger contract is the spender
        burn_token = ctx.usdc_token_address_on_source_chain

        payload = await ctx.rebalancer_contract.build_and_sign_cctp_approve_before_burn_tx(
            source_chain=ctx.from_chain_id,
            amount=ctx.amount + (ctx.cctp_fees or 0), # considering the fees
            spender=spender,
            to=burn_token
        )

        broadcast(ctx.web3_source, payload)

        print(f"Approved {ctx.amount + (ctx.cctp_fees or 0)} of token {burn_token} to spender {spender} on chainId={ctx.from_chain_id}")