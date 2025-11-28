import time
from near_omni_client.adapters.cctp.attestation_service import AttestationService
from ..strategy_context import StrategyContext
from .step import Step

class WaitAttestation(Step):
    NAME = "WaitAttestation"

    async def run(self, ctx: StrategyContext):
        attestation_service = AttestationService(ctx.from_network_id)

        attestation = attestation_service.retrieve_attestation(
            transaction_hash=ctx.burn_tx_hash
        )

        print("Attestation retrieved successfully!")
        
        ctx.attestation = attestation

        time.sleep(2)