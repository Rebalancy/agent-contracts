from helpers import broadcaster

class ApproveBeforeBurn:
    NAME = "ApproveBeforeBurn"

    async def run(self, ctx):
        spender = ctx.remote_config[ctx.from_chain_id]["cctp"]["messenger_address"]
        burn_token = ctx.usdc_token_address

        payload = await ctx.rebalancer_contract.build_and_sign_cctp_approve_before_burn_tx(
            source_chain=ctx.from_chain_id,
            amount=ctx.amount + (ctx.cctp_fees or 0),
            spender=spender,
            to=burn_token
        )

        broadcaster.broadcast(ctx.web3_source, payload)