from near_omni_client.json_rpc.client import NearClient
from web3 import Web3

from .strategy import Strategy
from ..rebalancer_contract import RebalancerContract
from ..types import Flow
# from .steps import step_cctp_burn, step_aave_supply, step_aave_withdraw, step_cctp_mint, step_rebalancer_deposit, step_rebalancer_withdraw_to_allocate, wait_for_attestation

class AaveToAave(Strategy):
    def __init__(self, *, rebalancer_contract: RebalancerContract) -> None:
        self.rebalancer_contract = rebalancer_contract

    async def execute(self, *, from_chain_id: int, to_chain_id: int, amount: int) -> None:
        print(f"🟦 Flow Rebalancer→Aave | from={from_chain_id} to={to_chain_id} amount={amount}")
        nonce = await self.rebalancer_contract.start_rebalance(flow=Flow.AaveToAave, source_chain=from_chain_id, destination_chain=to_chain_id, expected_amount=amount)
        withdraw_payload = await self.rebalancer_contract.build_withdraw_for_crosschain_allocation_tx(from_chain_id=from_chain_id, amount=amount)
        # await tx_propagator.send_raw_tx(withdraw_payload)
        # burn_payload = await self.rebalancer_contract.build_cctp_burn_tx(from_chain_id=from_chain_id, to_chain_id=to_chain_id, amount=amount, nonce=nonce)
        # att  = await wait_for_attestation(burn_tx_hash=burn, from_chain_id=from_chain_id,
        #                            to_chain_id=to_chain_id, min_finality_threshold=self.rebalancer_contract.min_finality)
        # mint_payload = await self.rebalancer_contract.build_cctp_mint
        # supply_payload = await self.rebalancer_contract.build_aave_supply_tx(to_chain_id=to_chain_id, amount=amount)
        print("✅ Done Aave→Aave\n")

