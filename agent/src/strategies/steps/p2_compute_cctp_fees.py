from near_omni_client.adapters.cctp.fee_service import FeeService

class ComputeCCTPFees:
    NAME = "ComputeCCTPFees"

    async def run(self, ctx):
        domain = int(ctx.to_network_id.domain)
        fee_service = FeeService(ctx.from_network_id)

        f = fee_service.get_fees(destination_domain_id=domain)
        base_fee = f.minimumFee

        raw_fees = int((base_fee * ctx.amount // 10_000) * 1.05)
        ctx.cctp_fees = min(raw_fees, ctx.config.max_bridge_fee)