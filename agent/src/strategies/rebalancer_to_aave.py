import time

from typing import Dict
from near_omni_client.providers.evm.alchemy_provider import AlchemyFactoryProvider
from near_omni_client.adapters.cctp.attestation_service import AttestationService

from .strategy import Strategy
from .broadcaster import broadcast
from config import Config
from rebalancer_contract import RebalancerContract
from tx_types import Flow
from utils import from_chain_id_to_network

class RebalancerToAave(Strategy):
    def __init__(self, *, rebalancer_contract: RebalancerContract, evm_factory_provider: AlchemyFactoryProvider, vault_address: str, config: Config, remote_config: Dict[str, dict]) -> None:
        self.rebalancer_contract = rebalancer_contract
        self.evm_factory_provider = evm_factory_provider
        self.vault_address = vault_address
        self.config = config
        self.remote_config = remote_config

    async def execute(self, *, from_chain_id: int, to_chain_id: int, amount: int) -> None:
        print(f"🟩 Flow Rebalancer→Aave | from={from_chain_id} to={to_chain_id} amount={amount}")
        network_id = from_chain_id_to_network(from_chain_id)
        to_network_id = from_chain_id_to_network(to_chain_id)
        web3_instance = self.evm_factory_provider.get_provider(network_id)
        web3_instance_destination_chain = self.evm_factory_provider.get_provider(to_network_id)

        # Step 1: Start Rebalance
        nonce = await self.rebalancer_contract.start_rebalance(flow=Flow.RebalancerToAave, source_chain=from_chain_id, destination_chain=to_chain_id, expected_amount=amount)
        print(f"Started rebalance with nonce: {nonce}")

        # Step 2: Withdraw from Rebalancer on source chain
        withdraw_payload = await self.rebalancer_contract.build_and_sign_withdraw_for_crosschain_allocation_tx(source_chain=from_chain_id, amount=amount, to=self.vault_address)

        try:
            broadcast(web3_instance, withdraw_payload)
        except Exception as e:
            print(f"Error broadcasting withdraw transaction: {e}")
            return
        
        # Step 3: Approve USDC for burn on source chain
        burn_token = self.remote_config[from_chain_id]["aave"]["asset"]
        spender = self.remote_config[from_chain_id]["cctp"]["messenger_address"] # the messenger contract is the spender
        messenger_contract_address = spender
        approve_payload = await self.rebalancer_contract.build_and_sign_cctp_approve_before_burn_tx(source_chain=from_chain_id, amount=amount, spender=spender, to=burn_token)
        try:
            broadcast(web3_instance, approve_payload)
        except Exception as e:
            print(f"Error broadcasting approve transaction: {e}")
            return
        
        time.sleep(3)
        
        # Step 4: Burn on source chain to initiate CCTP transfer
        print(f"Using burn token: {burn_token} on chainId={from_chain_id}")
        burn_payload = await self.rebalancer_contract.build_and_sign_cctp_burn_tx(source_chain=from_chain_id, to_chain_id=to_chain_id, amount=amount, burn_token=burn_token,to=messenger_contract_address) 
        try:
            burn_tx_hash = broadcast(web3_instance, burn_payload)
        except Exception as e:
            print(f"Error broadcasting burn transaction: {e}")
            return
    
        # Step 4: Wait for attestation and mint on destination chain
        print(f"Burn transaction hash: 0x{burn_tx_hash}")
        print("Getting the attestation...")

        attestation_service = AttestationService(from_chain_id_to_network(from_chain_id))
        attestation  = attestation_service.retrieve_attestation(transaction_hash=f"0x{burn_tx_hash}")
        print("Attestation retrieved successfully!")
        print(f"Attestation: {attestation}")
        transmitter_contract_address = self.remote_config[to_chain_id]["cctp"]["transmitter_address"]
        print(f"transmitter_contract_address: {transmitter_contract_address}")

        time.sleep(3)
        
        mint_payload = await self.rebalancer_contract.build_and_sign_cctp_mint_tx(to_chain_id=to_chain_id, message=attestation.message, attestation=attestation.attestation, to=transmitter_contract_address)
        try:
            broadcast(web3_instance_destination_chain, mint_payload)
        except Exception as e:
            print(f"Error broadcasting mint transaction: {e}")
            return

        print("Mint transaction broadcasted successfully!")
        time.sleep(3)

        # Step 5: Approve USDC before supply on destination chain
        usdc_on_destination_chain = self.remote_config[to_chain_id]["aave"]["asset"]
        spender = self.remote_config[to_chain_id]["aave"]["lending_pool_address"] # the lending pool is the spender
        lending_pool_address = spender
        approve_usdc_aave_payload = await self.rebalancer_contract.build_and_sign_aave_approve_before_supply_tx(to_chain_id=to_chain_id,amount=amount, spender=lending_pool_address, to=usdc_on_destination_chain)
        try:
            broadcast(web3_instance_destination_chain, approve_usdc_aave_payload)
        except Exception as e:
            print(f"Error broadcasting approve transaction: {e}")
            return
        
        print("USDC approved for Aave supply.")

        time.sleep(3)

        # Step 6: Deposit into Aave on destination chain
        asset = self.remote_config[to_chain_id]["aave"]["asset"]
        on_behalf_of = self.remote_config[to_chain_id]["aave"]["on_behalf_of"]
        referral_code = self.remote_config[to_chain_id]["aave"]["referral_code"]
        aave_lending_pool = self.remote_config[to_chain_id]["aave"]["lending_pool_address"]
        aave_supply_payload = await self.rebalancer_contract.build_and_sign_aave_supply_tx(to_chain_id=to_chain_id, asset=asset, amount=amount, on_behalf_of=on_behalf_of, referral_code=referral_code, to=aave_lending_pool)
       
        try:
            broadcast(web3_instance_destination_chain, aave_supply_payload)
        except Exception as e:
            print(f"Error broadcasting mint transaction: {e}")
            return
        
        print("Broadcasting supply transaction...")
        print("✅ Done Rebalancer→Aave\n")