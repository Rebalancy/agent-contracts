import time

from typing import Dict
from near_omni_client.providers.evm import AlchemyFactoryProvider
from near_omni_client.adapters.cctp.attestation_service import AttestationService

from .strategy import Strategy
from helpers import broadcast
from config import Config
from adapters import RebalancerContract
from utils import from_chain_id_to_network

class AaveToRebalancer(Strategy):
    def __init__(self, *, rebalancer_contract: RebalancerContract, evm_factory_provider: AlchemyFactoryProvider, vault_address: str, config: Config, remote_config: Dict[str, dict], agent_address: str) -> None:
        self.rebalancer_contract = rebalancer_contract
        self.evm_factory_provider = evm_factory_provider
        self.vault_address = vault_address
        self.config = config
        self.remote_config = remote_config
        self.agent_address = agent_address

    async def execute(self, *, from_chain_id: int, to_chain_id: int, amount: int) -> None:
        print(f"🟩 Flow Aave→Rebalancer | from={from_chain_id} to={to_chain_id} amount={amount}")
        network_id = from_chain_id_to_network(from_chain_id)
        to_network_id = from_chain_id_to_network(to_chain_id)
        
        web3_instance_source_chain = self.evm_factory_provider.get_provider(network_id)
        web3_instance_destination_chain = self.evm_factory_provider.get_provider(to_network_id)

        # TODO: Step 1: Withdraw from Aave on source chain           
        withdraw_payload = await self.rebalancer_contract.build_aave_withdraw_tx(from_chain_id=from_chain_id, amount=amount)

        try:
            broadcast(web3_instance_source_chain, withdraw_payload)
        except Exception as e:
            print(f"Error broadcasting withdraw transaction: {e}")
            return
        
        # Step 2: Approve USDC for burn on source chain
        burn_token = self.remote_config[from_chain_id]["aave"]["asset"]
        spender = self.remote_config[from_chain_id]["cctp"]["messenger_address"] # the messenger contract is the spender
        messenger_contract_address = spender
        approve_payload = await self.rebalancer_contract.build_and_sign_cctp_approve_before_burn_tx(source_chain=from_chain_id, amount=amount, spender=spender, to=burn_token)
        
        try:
            broadcast(web3_instance_source_chain, approve_payload)
        except Exception as e:
            print(f"Error broadcasting approve transaction: {e}")
            return
        
        time.sleep(3)

        # Step 3: Burn on source chain to initiate CCTP transfer
        print(f"Using burn token: {burn_token} on chainId={from_chain_id}")
        burn_payload = await self.rebalancer_contract.build_and_sign_cctp_burn_tx(source_chain=from_chain_id, to_chain_id=to_chain_id, amount=amount, burn_token=burn_token,to=messenger_contract_address) 
        try:
            burn_tx_hash = broadcast(web3_instance_source_chain, burn_payload)
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

        # TODO: Step 5: Deposit into Rebalancer on destination chain
        deposit_payload = await self.rebalancer_contract.build_rebalancer_deposit_tx(to_chain_id=to_chain_id, amount=amount)
        try:
            broadcast(web3_instance_destination_chain, deposit_payload)
        except Exception as e:
            print(f"Error broadcasting deposit transaction: {e}")
            return
        print("Deposit transaction broadcasted successfully!")

        print("✅ Done Aave→Rebalancer\n")


# from engine.phased_strategy import PhasedStrategy
# from engine.phases.setup_providers import SetupProvidersPhase
# from engine.phases.withdraw_from_aave import WithdrawFromAavePhase
# from engine.phases.approve_before_burn import ApproveBeforeBurnPhase
# from engine.phases.burn import BurnPhase
# from engine.phases.wait_attestation import WaitAttestationPhase
# from engine.phases.mint import MintPhase
# from engine.phases.deposit_into_rebalancer import DepositIntoRebalancerPhase

# class AaveToRebalancer(PhasedStrategy):
#     NAME = "Aave→Rebalancer"
#     PHASES = [
#         SetupProvidersPhase,
#         WithdrawFromAavePhase,
#         ApproveBeforeBurnPhase,
#         BurnPhase,
#         WaitAttestationPhase,
#         MintPhase,
#         DepositIntoRebalancerPhase,
#     ]