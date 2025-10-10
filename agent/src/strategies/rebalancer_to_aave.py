from typing import Dict
from near_omni_client.providers.evm.alchemy_provider import AlchemyFactoryProvider

from .strategy import Strategy
from .broadcaster import broadcast
from config import Config
from rebalancer_contract import RebalancerContract
from tx_types import Flow
from utils import address_to_bytes32, from_chain_id_to_network

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
        web3_instance = self.evm_factory_provider.get_provider(network_id)

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
        
        # Step 3: Burn on source chain to initiate CCTP transfer
        burn_token = self.remote_config[from_chain_id]["aave"]["asset"]
        print(f"Using burn token: {burn_token} on chainId={from_chain_id}")
        burn_payload = await self.rebalancer_contract.build_and_sign_cctp_burn_tx(source_chain=from_chain_id, to_chain_id=to_chain_id, amount=amount, burn_token=burn_token,to=self.vault_address)
        try:
            burn_tx_hash = broadcast(web3_instance, burn_payload)
        except Exception as e:
            print(f"Error broadcasting burn transaction: {e}")
            return
    
        # Step 4: Wait for attestation and mint on destination chain
        print(f"Burn transaction hash: {burn_tx_hash}")
        print("⏳ Waiting for attestation and mint on destination chain...")
        # Step 5: Deposit into Aave on destination chain
        # att  = await wait_for_attestation(burn_tx_hash=burn, from_chain_id=from_chain_id,
        #                            to_chain_id=to_chain_id, min_finality_threshold=self.rebalancer_contract.min_finality)
        # mint_payload = await self.rebalancer_contract.build_cctp_mint_tx(to_chain_id=to_chain_id, attestation_payload=att)
        # deposit_payload = await self.rebalancer_contract.build_rebalancer_deposit_tx(to_chain_id=to_chain_id, amount=amount)
        print("✅ Done Rebalancer→Aave\n")