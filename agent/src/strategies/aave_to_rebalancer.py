from typing import Dict
from near_omni_client.providers.evm import AlchemyFactoryProvider

from .strategy import Strategy
from .broadcaster import broadcast
from config import Config
from rebalancer_contract import RebalancerContract
from tx_types import Flow
from utils import from_chain_id_to_network

class AaveToRebalancer(Strategy):
    def __init__(self, *, rebalancer_contract: RebalancerContract, evm_factory_provider: AlchemyFactoryProvider, vault_address: str, config: Config, remote_config: Dict[str, dict]) -> None:
        self.rebalancer_contract = rebalancer_contract
        self.evm_factory_provider = evm_factory_provider
        self.vault_address = vault_address
        self.config = config
        self.remote_config = remote_config

    async def execute(self, *, from_chain_id: int, to_chain_id: int, amount: int) -> None:
        print(f"🟩 Flow Aave→Rebalancer | from={from_chain_id} to={to_chain_id} amount={amount}")
        nonce = await self.rebalancer_contract.start_rebalance(flow=Flow.AaveToRebalancer, source_chain=from_chain_id, destination_chain=to_chain_id, expected_amount=amount)
        withdraw_payload = await self.rebalancer_contract.build_aave_withdraw_tx(from_chain_id=from_chain_id, amount=amount)
        network_id = from_chain_id_to_network(from_chain_id)

        web3_instance = self.evm_factory_provider.get_provider(network_id)

        tx_hash = web3_instance.eth.send_raw_transaction(withdraw_payload)
        print(f"   - Withdraw tx sent on {network_id} (chainId={from_chain_id})")
        print(f"tx hash {tx_hash}")
        # await tx_propagator.send_raw_tx(withdraw_payload)
        # burn_payload = await self.rebalancer_contract.build_cctp_burn_tx(from_chain_id=from_chain_id, to_chain_id=to_chain_id, amount=amount, nonce=nonce)
        # att  = await wait_for_attestation(burn_tx_hash=burn, from_chain_id=from_chain_id,
        #                            to_chain_id=to_chain_id, min_finality_threshold=self.rebalancer_contract.min_finality)
        # mint_payload = await self.rebalancer_contract.build_cctp_mint_tx(to_chain_id=to_chain_id, attestation_payload=att)
        # deposit_payload = await self.rebalancer_contract.build_rebalancer_deposit_tx(to_chain_id=to_chain_id, amount=amount)
        print("✅ Done Aave→Rebalancer\n")