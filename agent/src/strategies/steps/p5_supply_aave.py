import time
from helpers import broadcast
from ..strategy_context import StrategyContext
from .step import Step

class SupplyAave(Step):
    NAME = "SupplyAave"

    async def run(self, ctx: StrategyContext) -> None:
        asset = ctx.usdc_token_address_on_destination_chain
        on_behalf = ctx.agent_address
        referral = ctx.remote_config[ctx.to_chain_id]["aave"]["referral_code"]

        supply_payload = await ctx.rebalancer_contract.build_and_sign_aave_supply_tx(
            to_chain_id=ctx.to_chain_id,
            asset=asset,
            amount=ctx.amount,
            on_behalf_of=on_behalf,
            referral_code=referral,
            to=ctx.aave_lending_pool_address_on_destination_chain
        )

        broadcast(ctx.web3_destination, supply_payload)

        print("Aave supply transaction broadcasted successfully.")

        time.sleep(2)