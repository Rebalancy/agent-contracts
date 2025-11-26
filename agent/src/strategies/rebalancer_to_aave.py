import time

from typing import Dict
from near_omni_client.providers.evm.alchemy_provider import AlchemyFactoryProvider
from near_omni_client.adapters.cctp import AttestationService, FeeService

from .strategy import Strategy
from helpers import broadcast, Assert, BalanceHelper
from config import Config
from adapters import LendingPool,RebalancerContract
from engine_types import Flow
from utils import from_chain_id_to_network

class RebalancerToAave(Strategy):
    def __init__(self, *, rebalancer_contract: RebalancerContract, evm_factory_provider: AlchemyFactoryProvider, vault_address: str, config: Config, remote_config: Dict[str, dict], agent_address: str) -> None:
        self.rebalancer_contract = rebalancer_contract
        self.evm_factory_provider = evm_factory_provider
        self.vault_address = vault_address
        self.config = config
        self.remote_config = remote_config
        self.agent_address = agent_address

    async def execute(self, *, from_chain_id: int, to_chain_id: int, amount: int) -> None:
        print(f"🟩 Flow Rebalancer→Aave | from={from_chain_id} to={to_chain_id} amount={amount}")

        # Step 0: setup web3 instances, networks and extract information from remote config
        from_network_id = from_chain_id_to_network(from_chain_id)
        to_network_id = from_chain_id_to_network(to_chain_id)
        web3_instance_source_chain = self.evm_factory_provider.get_provider(from_network_id)
        web3_instance_destination_chain = self.evm_factory_provider.get_provider(to_network_id)

        burn_token = self.remote_config[from_chain_id]["aave"]["asset"]
        spender = self.remote_config[from_chain_id]["cctp"]["messenger_address"] # the messenger contract is the spender
        transmitter_contract_address = self.remote_config[to_chain_id]["cctp"]["transmitter_address"]
        messenger_contract_address = spender
        usdc_token_address = burn_token

        # Step 1: Start Rebalance
        nonce = await self.rebalancer_contract.start_rebalance(
            flow=Flow.RebalancerToAave, 
            source_chain=from_chain_id, 
            destination_chain=to_chain_id, 
            expected_amount=amount
        )
        print(f"Started rebalance with nonce: {nonce}")

        usdc_agent_balance_before = BalanceHelper.get_usdc_agent_balance(web3_instance_source_chain, usdc_token_address)

        # Step 2: Withdraw from Rebalancer on source chain
        withdraw_payload = await self.rebalancer_contract.build_and_sign_withdraw_for_crosschain_allocation_tx(
            source_chain=from_chain_id, 
            amount=amount, 
            to=self.vault_address
        )

        try:
            broadcast(web3_instance_source_chain, withdraw_payload)
        except Exception as e:
            print(f"Error broadcasting withdraw transaction: {e}")
            return
        
        # Step 2: Assert balance is correct after withdraw considering previous balance
        Assert.usdc_agent_balance(web3_instance_source_chain, usdc_token_address, expected_balance=amount + usdc_agent_balance_before)

        # Step 3: Approve USDC for burn on source chain
        destination_domain = int(to_network_id.domain)
        fee_service = FeeService(from_network_id)
        cctp_fees_typed = fee_service.get_fees(destination_domain_id=destination_domain)
        cctp_minimum_fee = cctp_fees_typed.minimumFee
        print(f"CCTP minimum fee for destination domain {destination_domain}: {cctp_minimum_fee}")
        cctp_fees = int((cctp_minimum_fee * amount // 10_000) * 1.05) # assuming BPS is in basis points (1/100 of a percent) + a 5% buffer
        print(f"CCTP fees for amount {amount}: {cctp_fees}")

        if cctp_fees > self.config.max_bridge_fee:
            cctp_fees = self.config.max_bridge_fee
            print(f"CCTP fees capped to max bridge fee: {cctp_fees}")

        approve_payload = await self.rebalancer_contract.build_and_sign_cctp_approve_before_burn_tx(
            source_chain=from_chain_id, 
            amount=amount + cctp_fees, # considering the fees
            spender=spender,
            to=burn_token
        )
        try:
            broadcast(web3_instance_source_chain, approve_payload)
        except Exception as e:
            print(f"Error broadcasting approve transaction: {e}")
            return
                
        # Step 4: Burn on source chain to initiate CCTP transfer
        print(f"Using burn token: {burn_token} on chainId={from_chain_id}")
       
        burn_payload = await self.rebalancer_contract.build_and_sign_cctp_burn_tx(
            source_chain=from_chain_id, 
            to_chain_id=to_chain_id, 
            amount=amount + cctp_fees,  # considering the fees
            max_fee=cctp_fees,
            burn_token=burn_token,
            to=messenger_contract_address
        )

        try:
            burn_tx_hash = broadcast(web3_instance_source_chain, burn_payload)
        except Exception as e:
            print(f"Error broadcasting burn transaction: {e}")
            return

        # Step 4: Assert balance is (balance before - fees)
        Assert.usdc_agent_balance(web3_instance_source_chain, usdc_token_address, expected_balance=usdc_agent_balance_before - cctp_fees)

        # Step 5: Wait for attestation
        print(f"Burn transaction hash: 0x{burn_tx_hash}")
        print("Getting the attestation...")

        attestation_service = AttestationService(from_network_id)
        attestation  = attestation_service.retrieve_attestation(transaction_hash=f"0x{burn_tx_hash}")
        
        print("Attestation retrieved successfully!")
        print(f"Attestation: {attestation}")

        time.sleep(3)
        
        # Step 6: Mint on destination chain
        mint_payload = await self.rebalancer_contract.build_and_sign_cctp_mint_tx(
            to_chain_id=to_chain_id, 
            message=attestation.message, 
            attestation=attestation.attestation, 
            to=transmitter_contract_address
        )

        try:
            broadcast(web3_instance_destination_chain, mint_payload)
        except Exception as e:
            print(f"Error broadcasting mint transaction: {e}")
            return

        print("Mint transaction broadcasted successfully!")

        time.sleep(5)

        # Step 6: Assert balance is correct after mint
        usdc_address_on_destination_chain = self.remote_config[to_chain_id]["aave"]["asset"]
        print(f"USDC on destination chain: {usdc_address_on_destination_chain}")
        Assert.usdc_agent_balance_is_at_least(web3_instance_destination_chain, usdc_address_on_destination_chain, expected_balance=amount)

        # Step 7: Approve USDC before supply on destination chain
        aave_lending_pool_address = self.remote_config[to_chain_id]["aave"]["lending_pool_address"] # the lending pool is the spender
        spender = aave_lending_pool_address
        approve_usdc_aave_payload = await self.rebalancer_contract.build_and_sign_aave_approve_before_supply_tx(
            to_chain_id=to_chain_id,
            amount=int(amount * 2), 
            spender=aave_lending_pool_address, 
            to=usdc_address_on_destination_chain
        )
        try:
            broadcast(web3_instance_destination_chain, approve_usdc_aave_payload)
        except Exception as e:
            print(f"Error broadcasting approve transaction: {e}")
            return
        
        print("USDC approved for Aave supply.")

        # Step 8: Deposit into Aave on destination chain
        asset = self.remote_config[to_chain_id]["aave"]["asset"]
        on_behalf_of = self.agent_address
        referral_code = self.remote_config[to_chain_id]["aave"]["referral_code"]
        aave_supply_payload = await self.rebalancer_contract.build_and_sign_aave_supply_tx(to_chain_id=to_chain_id, asset=asset, amount=amount, on_behalf_of=on_behalf_of, referral_code=referral_code, to=aave_lending_pool_address)

        a_token_address = LendingPool.get_atoken_address(web3_instance_destination_chain, aave_lending_pool_address, asset)
        a_token_balance_before = BalanceHelper.get_atoken_agent_balance(web3_instance_destination_chain, a_token_address)

        try:
            broadcast(web3_instance_destination_chain, aave_supply_payload)
        except Exception as e:
            print(f"Error broadcasting supply transaction: {e}")
            return
        
        time.sleep(3)

        # Step 8: Assert aToken balance is correct after supply and also considers previous balance
        Assert.atoken_agent_balance(web3_instance_destination_chain, a_token_address, expected_balance=amount + a_token_balance_before)
        
        print("Broadcasting supply transaction...")
        print("✅ Done Rebalancer→Aave\n")