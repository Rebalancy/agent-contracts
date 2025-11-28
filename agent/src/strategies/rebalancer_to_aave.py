from .strategy import Strategy
from .steps.p0_start_rebalance import StartRebalance
from .steps.p1_withdraw_from_rebalancer import WithdrawFromRebalancer
from .steps.p1_withdraw_from_rebalancer_after_assertion import WithdrawFromRebalancerAfterAssertion
from .steps.p0_get_usdc_agent_balance_before_rebalance import GetUSDCBalanceBeforeRebalance
from .steps.p2_compute_cctp_fees import ComputeCctpFees
from .steps.p2_approve_before_cctp_burn import ApproveBeforeCctpBurn
from .steps.p2_cctp_burn import CctpBurn
from .steps.p2_cctp_burn_after_assertion import CctpBurnAfterAssertion

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
        # WaitAttestationPhase,
        # MintPhase,
        # AssertPostMintBalancePhase,
        # ApproveAavePhase,
        # SupplyAavePhase,
    ]

   
    #     # Step 5: Wait for attestation
    #     print(f"Burn transaction hash: 0x{burn_tx_hash}")
    #     print("Getting the attestation...")

    #     attestation_service = AttestationService(from_network_id)
    #     attestation  = attestation_service.retrieve_attestation(transaction_hash=f"0x{burn_tx_hash}")
        
    #     print("Attestation retrieved successfully!")
    #     print(f"Attestation: {attestation}")

    #     time.sleep(3)
        
    #     # Step 6: Mint on destination chain
    #     mint_payload = await self.rebalancer_contract.build_and_sign_cctp_mint_tx(
    #         to_chain_id=to_chain_id, 
    #         message=attestation.message, 
    #         attestation=attestation.attestation, 
    #         to=transmitter_contract_address
    #     )

    #     try:
    #         broadcast(web3_instance_destination_chain, mint_payload)
    #     except Exception as e:
    #         print(f"Error broadcasting mint transaction: {e}")
    #         return

    #     print("Mint transaction broadcasted successfully!")

    #     time.sleep(5)

    #     # Step 6: Assert balance is correct after mint
    #     usdc_address_on_destination_chain = self.remote_config[to_chain_id]["aave"]["asset"]
    #     print(f"USDC on destination chain: {usdc_address_on_destination_chain}")
    #     Assert.usdc_agent_balance_is_at_least(web3_instance_destination_chain, usdc_address_on_destination_chain, expected_balance=amount)

    #     # Step 7: Approve USDC before supply on destination chain
    #     aave_lending_pool_address = self.remote_config[to_chain_id]["aave"]["lending_pool_address"] # the lending pool is the spender
    #     spender = aave_lending_pool_address
    #     approve_usdc_aave_payload = await self.rebalancer_contract.build_and_sign_aave_approve_before_supply_tx(
    #         to_chain_id=to_chain_id,
    #         amount=int(amount * 2), 
    #         spender=aave_lending_pool_address, 
    #         to=usdc_address_on_destination_chain
    #     )
    #     try:
    #         broadcast(web3_instance_destination_chain, approve_usdc_aave_payload)
    #     except Exception as e:
    #         print(f"Error broadcasting approve transaction: {e}")
    #         return
        
    #     print("USDC approved for Aave supply.")

    #     # Step 8: Deposit into Aave on destination chain
    #     asset = self.remote_config[to_chain_id]["aave"]["asset"]
    #     on_behalf_of = self.agent_address
    #     referral_code = self.remote_config[to_chain_id]["aave"]["referral_code"]
    #     aave_supply_payload = await self.rebalancer_contract.build_and_sign_aave_supply_tx(to_chain_id=to_chain_id, asset=asset, amount=amount, on_behalf_of=on_behalf_of, referral_code=referral_code, to=aave_lending_pool_address)

    #     a_token_address = LendingPool.get_atoken_address(web3_instance_destination_chain, aave_lending_pool_address, asset)
    #     a_token_balance_before = BalanceHelper.get_atoken_agent_balance(web3_instance_destination_chain, a_token_address)

    #     try:
    #         broadcast(web3_instance_destination_chain, aave_supply_payload)
    #     except Exception as e:
    #         print(f"Error broadcasting supply transaction: {e}")
    #         return
        
    #     time.sleep(3)

    #     # Step 8: Assert aToken balance is correct after supply and also considers previous balance
    #     Assert.atoken_agent_balance(web3_instance_destination_chain, a_token_address, expected_balance=amount + a_token_balance_before)
        
    #     print("Broadcasting supply transaction...")
    #     print("✅ Done Rebalancer→Aave\n")

