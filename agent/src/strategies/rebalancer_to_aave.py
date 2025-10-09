from near_omni_client.providers.evm.alchemy_provider import AlchemyFactoryProvider

from .strategy import Strategy
from .broadcaster import broadcast
from rebalancer_contract import RebalancerContract
from tx_types import Flow
from config import Config
from utils import from_chain_id_to_network

class RebalancerToAave(Strategy):
    def __init__(self, *, rebalancer_contract: RebalancerContract, evm_factory_provider: AlchemyFactoryProvider, vault_address: str, config: Config) -> None:
        self.rebalancer_contract = rebalancer_contract
        self.evm_factory_provider = evm_factory_provider
        self.vault_address = vault_address
        self.config = config

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
        # TODO: De donde saco el burn_token? que es USDC 
        burn_payload = await self.rebalancer_contract.build_and_sign_cctp_burn_tx(source_chain=from_chain_id, to_chain_id=to_chain_id, amount=amount, burn_token=self.vault_address)
        try:
            burn_tx_hash = broadcast(web3_instance, burn_payload)
        except Exception as e:
            print(f"Error broadcasting burn transaction: {e}")
            return
    
        # Step 4: Wait for attestation and mint on destination chain
        print(f"Burn transaction hash: {burn_tx_hash.hex()}")
        print("⏳ Waiting for attestation and mint on destination chain...")
        # Step 5: Deposit into Aave on destination chain
        # att  = await wait_for_attestation(burn_tx_hash=burn, from_chain_id=from_chain_id,
        #                            to_chain_id=to_chain_id, min_finality_threshold=self.rebalancer_contract.min_finality)
        # mint_payload = await self.rebalancer_contract.build_cctp_mint_tx(to_chain_id=to_chain_id, attestation_payload=att)
        # deposit_payload = await self.rebalancer_contract.build_rebalancer_deposit_tx(to_chain_id=to_chain_id, amount=amount)
        print("✅ Done Rebalancer→Aave\n")