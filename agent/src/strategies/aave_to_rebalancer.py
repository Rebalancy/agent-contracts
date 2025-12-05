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
from .steps.p6_complete_rebalance import CompleteRebalance
from .steps.p8_approve_vault_to_spend_agent_usdc import ApproveVaulToSpendAgentUSDC
from .steps.p9_get_usdc_balance_before_deposit_to_rebalancer import GetUSDCBalanceBeforeDepositToRebalancer 
from .steps.p7_deposit_into_rebalancer import DepositIntoRebalancer
from .steps.p7_deposit_into_rebalancer_after_assertion import DepositIntoRebalancerAfterAssertion

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
        ApproveVaulToSpendAgentUSDC,  
        GetUSDCBalanceBeforeDepositToRebalancer,
        DepositIntoRebalancer,
        DepositIntoRebalancerAfterAssertion,
        CompleteRebalance
    ]

