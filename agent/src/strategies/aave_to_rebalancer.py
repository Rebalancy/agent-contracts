from .strategy import Strategy
from .steps.p0_start_rebalance import StartRebalance
from .steps.p6_withdraw_from_aave import WithdrawFromAave
from .steps.p6_withdraw_from_aave_after_assertion import WithdrawFromAaveAfterAssertion
from .steps.p0_get_usdc_agent_balance_before_rebalance import GetUSDCBalanceBeforeRebalance
from .steps.p2_compute_cctp_fees import ComputeCctpFees
from .steps.p2_approve_before_cctp_burn import ApproveBeforeCctpBurn
from .steps.p2_cctp_burn import CctpBurn
from .steps.p2_cctp_burn_after_assertion import CctpBurnAfterAssertion
from .steps.p3_wait_attestation import WaitAttestation
from .steps.p4_cctp_mint import CttpMint
from .steps.p4_cctp_mint_after_assertion import CttpMintAfterAssertion

class AaveToRebalancer(Strategy):
    NAME = "Aave→Rebalancer"
    STEPS = [
        GetUSDCBalanceBeforeRebalance,
        StartRebalance,
        WithdrawFromAave,
        WithdrawFromAaveAfterAssertion,
        ComputeCctpFees,
        ApproveBeforeCctpBurn,
        CctpBurn,
        CctpBurnAfterAssertion,
        WaitAttestation,
        CttpMint,
        CttpMintAfterAssertion,
        # ApproveAaveUSDCBeforeSupply, TODO: Reemplazar por global approval
        # GetATokenBalanceBeforeSupply, # TODO: Esto deberia ser get usdc token balance before deposit to rebalancer
        # SupplyAave, # TODO: Reemplazar por deposit to rebalancer
        # SupplyAaveAfterAssertion # TODO: Reemplazar por deposit to rebalancer after assertion
    ]

#         # TODO: Step 1: Withdraw from Aave on source chain           
#         withdraw_payload = await self.rebalancer_contract.build_aave_withdraw_tx(from_chain_id=from_chain_id, amount=amount)

#         # TODO: Step 5: Deposit into Rebalancer on destination chain
#         deposit_payload = await self.rebalancer_contract.build_rebalancer_deposit_tx(to_chain_id=to_chain_id, amount=amount)
#         try:
#             broadcast(web3_instance_destination_chain, deposit_payload)
#         except Exception as e:
#             print(f"Error broadcasting deposit transaction: {e}")
#             return
#         print("Deposit transaction broadcasted successfully!")

#         print("✅ Done Aave→Rebalancer\n")
