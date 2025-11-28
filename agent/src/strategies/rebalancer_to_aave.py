from .strategy import Strategy
from .steps.p0_start_rebalance import StartRebalance
from .steps.p1_withdraw_from_rebalancer import WithdrawFromRebalancer
from .steps.p1_withdraw_from_rebalancer_after_assertion import WithdrawFromRebalancerAfterAssertion
from .steps.p0_get_usdc_agent_balance_before_rebalance import GetUSDCBalanceBeforeRebalance
from .steps.p2_compute_cctp_fees import ComputeCctpFees
from .steps.p2_approve_before_cctp_burn import ApproveBeforeCctpBurn
from .steps.p2_cctp_burn import CctpBurn
from .steps.p2_cctp_burn_after_assertion import CctpBurnAfterAssertion
from .steps.p3_wait_attestation import WaitAttestation
from .steps.p4_cctp_mint import CttpMint
from .steps.p4_cctp_mint_after_assertion import CttpMintAfterAssertion
from .steps.p5_approve_aave_before_supply import ApproveAaveUSDCBeforeSupply
from .steps.p5_get_a_token_agent_balance_before_supply import GetATokenBalanceBeforeSupply
from .steps.p5_supply_aave import SupplyAave
from .steps.p5_supply_aave_after_assertion import SupplyAaveAfterAssertion
from steps.p6_complete_rebalance import CompleteRebalance

class RebalancerToAave(Strategy):
    NAME = "Rebalancer→Aave"
    STEPS = [
        GetUSDCBalanceBeforeRebalance,
        StartRebalance,
        WithdrawFromRebalancer,
        WithdrawFromRebalancerAfterAssertion,
        ComputeCctpFees,
        ApproveBeforeCctpBurn,
        CctpBurn,
        CctpBurnAfterAssertion,
        WaitAttestation,
        CttpMint,
        CttpMintAfterAssertion,
        ApproveAaveUSDCBeforeSupply,
        GetATokenBalanceBeforeSupply,
        SupplyAave,
        SupplyAaveAfterAssertion,
        CompleteRebalance,
    ]