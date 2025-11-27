from helpers import state_assertions

class AssertAfterMint:
    NAME = "AssertAfterMint"

    async def run(self, ctx):
        usdc_dest = ctx.remote_config[ctx.to_chain_id]["aave"]["asset"]
        ctx.usdc_address_on_destination_chain = usdc_dest

        state_assertions.usdc_agent_balance_is_at_least(
            ctx.web3_destination,
            usdc_dest,
            expected_balance=ctx.amount
        )