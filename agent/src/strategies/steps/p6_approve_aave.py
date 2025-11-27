from helpers import broadcaster

class ApproveAave:
    NAME = "ApproveAave"

    async def run(self, ctx):
        spender = ctx.remote_config[ctx.to_chain_id]["aave"]["lending_pool_address"]
        ctx.aave_lending_pool_address = spender

        payload = await ctx.rebalancer_contract.build_and_sign_aave_approve_before_supply_tx(
            to_chain_id=ctx.to_chain_id,
            amount=int(ctx.amount * 2),
            spender=spender,
            to=ctx.usdc_address_on_destination_chain
        )

        broadcaster.broadcast(ctx.web3_destination, payload)