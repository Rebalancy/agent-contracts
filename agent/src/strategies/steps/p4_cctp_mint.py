import time
from helpers import broadcast
from ..strategy_context import StrategyContext
from .step import Step

class CttpMint(Step):
    NAME = "CttpMint"

    async def run(self, ctx: StrategyContext):
        payload = await ctx.rebalancer_contract.build_and_sign_cctp_mint_tx(
            to_chain_id=ctx.to_chain_id,
            message=ctx.attestation.message,
            attestation=ctx.attestation.attestation,
            to=ctx.transmitter_address_on_destination_chain
        )

        broadcast(ctx.web3_destination, payload)

        print("Mint transaction broadcasted successfully!")

        time.sleep(3)