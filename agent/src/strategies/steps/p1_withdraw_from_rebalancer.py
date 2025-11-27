from helpers import balance_helper, broadcaster

class WithdrawFromRebalancer:
    NAME = "WithdrawFromRebalancer"

    async def run(self, ctx):
        burn_token = ctx.remote_config[ctx.from_chain_id]["aave"]["asset"]
        ctx.usdc_token_address = burn_token

        before = balance_helper.get_usdc_agent_balance(ctx.web3_source, burn_token)
        ctx.usdc_agent_balance_before = before

        payload = await ctx.rebalancer_contract.build_and_sign_withdraw_for_crosschain_allocation_tx(
            source_chain=ctx.from_chain_id,
            amount=ctx.amount,
            to=ctx.vault_address
        )
        broadcaster.broadcast(ctx.web3_source, payload)