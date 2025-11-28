from helpers import broadcast
from ..strategy_context import StrategyContext
from .step import Step

class CctpBurn(Step):
    NAME = "CctpBurn"

    async def run(self, ctx: StrategyContext):
        burn_token = ctx.usdc_token_address_on_source_chain

        payload = await ctx.rebalancer_contract.build_and_sign_cctp_burn_tx(
            source_chain=ctx.from_chain_id,
            to_chain_id=ctx.to_chain_id,
            amount=ctx.amount + (ctx.cctp_fees or 0),
            max_fee=ctx.cctp_fees or 0,
            burn_token=burn_token,
            to=ctx.messenger_address_on_source_chain
        )

        tx_hash = broadcast(ctx.web3_source, payload)

        print(f"Burn transaction broadcasted successfully!")

        ctx.burn_tx_hash = f"0x{tx_hash}"