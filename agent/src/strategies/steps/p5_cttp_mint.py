from helpers import broadcaster

class CttpMint:
    NAME = "CttpMint"

    async def run(self, ctx):
        transmitter = ctx.remote_config[ctx.to_chain_id]["cctp"]["transmitter_address"]

        payload = await ctx.rebalancer_contract.build_and_sign_cctp_mint_tx(
            to_chain_id=ctx.to_chain_id,
            message=ctx.attestation.message,
            attestation=ctx.attestation.attestation,
            to=transmitter
        )

        broadcaster.broadcast(ctx.web3_destination, payload)