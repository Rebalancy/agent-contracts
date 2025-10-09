from typing import Dict
from near_omni_client.providers.evm import AlchemyFactoryProvider

from .strategy import Strategy
from .broadcaster import broadcast
from config import Config
from rebalancer_contract import RebalancerContract
from tx_types import Flow
from utils import from_chain_id_to_network

class AaveToAave(Strategy):
    def __init__(self, *, rebalancer_contract: RebalancerContract, evm_factory_provider: AlchemyFactoryProvider, vault_address: str, config: Config, remote_config: Dict[str, dict]) -> None:
        self.rebalancer_contract = rebalancer_contract
        self.evm_factory_provider = evm_factory_provider
        self.vault_address = vault_address
        self.config = config
        self.remote_config = remote_config

    async def execute(self, *, from_chain_id: int, to_chain_id: int, amount: int) -> None:
        print(f"🟦 Flow Rebalancer→Aave | from={from_chain_id} to={to_chain_id} amount={amount}")
        nonce = await self.rebalancer_contract.start_rebalance(flow=Flow.AaveToAave, source_chain=from_chain_id, destination_chain=to_chain_id, expected_amount=amount)
        withdraw_payload = await self.rebalancer_contract.build_withdraw_for_crosschain_allocation_tx(from_chain_id=from_chain_id, amount=amount)
        # await tx_propagator.send_raw_tx(withdraw_payload)
        # burn_payload = await self.rebalancer_contract.build_cctp_burn_tx(from_chain_id=from_chain_id, to_chain_id=to_chain_id, amount=amount, nonce=nonce)
        # att  = await wait_for_attestation(burn_tx_hash=burn, from_chain_id=from_chain_id,
        #                            to_chain_id=to_chain_id, min_finality_threshold=self.rebalancer_contract.min_finality)
        # mint_payload = await self.rebalancer_contract.build_cctp_mint
        # supply_payload = await self.rebalancer_contract.build_aave_supply_tx(to_chain_id=to_chain_id, amount=amount)
        print("✅ Done Aave→Aave\n")

