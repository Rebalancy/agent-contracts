import time
from near_omni_client.adapters.cctp.attestation_service import AttestationService

class WaitAttestation:
    NAME = "WaitAttestation"

    async def run(self, ctx):
        attestation_service = AttestationService(ctx.from_network_id)

        # blocking call (sync)
        attestation = attestation_service.retrieve_attestation(
            transaction_hash=ctx.burn_tx_hash
        )
        ctx.attestation = attestation

        time.sleep(2)