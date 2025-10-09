from near_omni_client.providers.evm import AlchemyFactoryProvider
from near_omni_client.json_rpc.client import NearClient

from .strategy import Strategy
from rebalancer_contract import RebalancerContract
from tx_types import Flow
from config import Config
from utils import from_chain_id_to_network
from near_omni_client.providers.evm.alchemy_provider import AlchemyFactoryProvider

class RebalancerToAave(Strategy):
    def __init__(self, *, rebalancer_contract: RebalancerContract, evm_factory_provider: AlchemyFactoryProvider, vault_address: str, config: Config) -> None:
        self.rebalancer_contract = rebalancer_contract
        self.evm_factory_provider = evm_factory_provider
        self.vault_address = vault_address
        self.config = config

    async def execute(self, *, from_chain_id: int, to_chain_id: int, amount: int) -> None:
        print(f"🟩 Flow Rebalancer→Aave | from={from_chain_id} to={to_chain_id} amount={amount}")
        nonce = await self.rebalancer_contract.start_rebalance(flow=Flow.RebalancerToAave, source_chain=from_chain_id, destination_chain=to_chain_id, expected_amount=amount)
        withdraw_payload = await self.rebalancer_contract.build_and_sign_withdraw_for_crosschain_allocation_tx(source_chain=from_chain_id, amount=amount, to=self.vault_address)
        network_id = from_chain_id_to_network(from_chain_id)

        web3_instance = self.evm_factory_provider.get_provider(network_id)
        try:
            if not web3_instance:
                raise ValueError("Web3 provider is not initialized.")
        except Exception as e:
            print(f"Error getting web3 instance: {e}")
            return
        
        try:
            tx_hash = web3_instance.eth.send_raw_transaction(withdraw_payload)
            print(f"tx_hash (hex) 0x{tx_hash.hex()}")
        except Exception as e:
            print(f"Error sending raw transaction: {e}")
            return

        # TODO: Fix que es lo que paso aqui o sea los args
        burn_payload = await self.rebalancer_contract.build_and_sign_cctp_burn_tx(from_chain_id=from_chain_id, to_chain_id=to_chain_id, amount=amount, nonce=nonce)
        try:
            tx_hash = web3_instance.eth.send_raw_transaction(burn_payload)
            print(f"tx_hash (hex) 0x{tx_hash.hex()}")
        except Exception as e:
            print(f"Error sending raw transaction: {e}")
            return

        # att  = await wait_for_attestation(burn_tx_hash=burn, from_chain_id=from_chain_id,
        #                            to_chain_id=to_chain_id, min_finality_threshold=self.rebalancer_contract.min_finality)
        # mint_payload = await self.rebalancer_contract.build_cctp_mint_tx(to_chain_id=to_chain_id, attestation_payload=att)
        # deposit_payload = await self.rebalancer_contract.build_rebalancer_deposit_tx(to_chain_id=to_chain_id, amount=amount)
        print("✅ Done Rebalancer→Aave\n")