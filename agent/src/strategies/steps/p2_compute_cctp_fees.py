from near_omni_client.adapters.cctp.fee_service import FeeService
from ..strategy_context import StrategyContext
from .step import Step

BUFFER = 1.05  # 5% buffer

class ComputeCctpFees(Step):
    NAME = "ComputeCctpFees"

    async def run(self, ctx: StrategyContext):
        domain = int(ctx.to_network_id.domain)
        fee_service = FeeService(ctx.from_network_id)

        cctp_fees_typed = fee_service.get_fees(destination_domain_id=domain)
        cctp_minimum_fee = cctp_fees_typed.minimumFee

        print(f"CCTP minimum fee for destination domain {domain}: {cctp_minimum_fee}")

        raw_fees = int((cctp_minimum_fee * ctx.amount // 10_000) * BUFFER)
        print(f"CCTP fees for amount {ctx.amount}: {raw_fees}")

        ctx.cctp_fees = min(raw_fees, ctx.config.max_bridge_fee) # cap to max bridge fee