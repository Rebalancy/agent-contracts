import time
from helpers import broadcaster, balance_helper
from adapters.lending_pool_contract import LendingPool

class SupplyAave:
    NAME = "SupplyAave"

    async def run(self, ctx):
        asset = ctx.remote_config[ctx.to_chain_id]["aave"]["asset"]
        lp = ctx.aave_lending_pool_address
        on_behalf = ctx.agent_address
        referral = ctx.remote_config[ctx.to_chain_id]["aave"]["referral_code"]

        payload = await ctx.rebalancer_contract.build_and_sign_aave_supply_tx(
            to_chain_id=ctx.to_chain_id,
            asset=asset,
            amount=ctx.amount,
            on_behalf_of=on_behalf,
            referral_code=referral,
            to=lp
        )

        a_token = LendingPool.get_atoken_address(ctx.web3_destination, lp, asset)
        before = balance_helper.get_atoken_agent_balance(ctx.web3_destination, a_token)
        ctx.a_token_address = a_token

        broadcaster.broadcast(ctx.web3_destination, payload)

        time.sleep(2)

        from helpers import state_assertions
        state_assertions.atoken_agent_balance(
            ctx.web3_destination, a_token,
            before + ctx.amount
        )