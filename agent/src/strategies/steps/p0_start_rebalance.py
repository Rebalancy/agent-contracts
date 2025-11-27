class StartRebalance:
    NAME = "StartRebalance"

    async def run(self, ctx):
        ctx.nonce = await ctx.rebalancer_contract.start_rebalance(
            flow="RebalancerToAave",
            source_chain=ctx.from_chain_id,
            destination_chain=ctx.to_chain_id,
            expected_amount=ctx.amount
        )