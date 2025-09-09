from near_omni_client.json_rpc.client import NearClient
from web3 import Web3

from .strategy import Strategy
from ..rebalancer_contract import RebalancerContract
from ..types import Flow
# async def execute_rebalancer_to_aave_flow(
#     *,
#     from_chain_id: int,
#     to_chain_id: int,
#     amount: int,
#     near_client: NearClient,
#     near_contract_id: str,
#     evm_factory_provider,
#     agent_address: str,
#     max_bridge_fee: int,
#     min_finality_threshold: int,
# ):
#     print(f"🟦 Flow Rebalancer→Aave | from={from_chain_id} to={to_chain_id} amount={amount}")
#     await step_rebalancer_withdraw_to_allocate(
#         from_chain_id=from_chain_id, amount=amount,
#         near_client=near_client, near_contract_id=near_contract_id, evm_factory_provider=evm_factory_provider
#     )
    # burn_hash = await step_cctp_burn(
    #     from_chain_id=from_chain_id, to_chain_id=to_chain_id, amount=amount,
    #     agent_address=agent_address, max_bridge_fee=max_bridge_fee, min_finality_threshold=min_finality_threshold,
    #     near_client=near_client, near_contract_id=near_contract_id, evm_factory_provider=evm_factory_provider
    # )
    # att = await _wait_attestation(burn_hash, from_chain_id, to_chain_id, min_finality_threshold)
    # await step_cctp_mint(
    #     to_chain_id=to_chain_id, attestation_payload=att,
    #     near_client=near_client, near_contract_id=near_contract_id, evm_factory_provider=evm_factory_provider
    # )
    # await step_aave_supply(
    #     to_chain_id=to_chain_id, amount=amount,
    #     near_client=near_client, near_contract_id=near_contract_id, evm_factory_provider=evm_factory_provider
    # )
    # print(" ✅ Flow Rebalancer→Aave completed.\n")

# async def execute_aave_to_rebalancer_flow(
#     *,
#     from_chain_id: int,
#     to_chain_id: int,
#     amount: int,
#     near_client: NearClient,
#     near_contract_id: str,
#     evm_factory_provider,
#     agent_address: str,
#     max_bridge_fee: int,
#     min_finality_threshold: int,
# ):
#     print(f"🟩 Flow Aave→Rebalancer | from={from_chain_id} to={to_chain_id} amount={amount}")
    # await step_aave_withdraw(
    #     from_chain_id=from_chain_id, amount=amount,
    #     near_client=near_client, near_contract_id=near_contract_id, evm_factory_provider=evm_factory_provider
    # )
    # burn_hash = await step_cctp_burn(
    #     from_chain_id=from_chain_id, to_chain_id=to_chain_id, amount=amount,
    #     agent_address=agent_address, max_bridge_fee=max_bridge_fee, min_finality_threshold=min_finality_threshold,
    #     near_client=near_client, near_contract_id=near_contract_id, evm_factory_provider=evm_factory_provider
    # )
    # att = await _wait_attestation(burn_hash, from_chain_id, to_chain_id, min_finality_threshold)
    # await step_cctp_mint(
    #     to_chain_id=to_chain_id, attestation_payload=att,
    #     near_client=near_client, near_contract_id=near_contract_id, evm_factory_provider=evm_factory_provider
    # )
    # await step_rebalancer_deposit(
    #     to_chain_id=to_chain_id, amount=amount,
    #     near_client=near_client, near_contract_id=near_contract_id, evm_factory_provider=evm_factory_provider
    # )
    # print(" ✅ Flow Aave→Rebalancer completed.\n")

# async def execute_aave_to_aave_flow(
#     *,
#     from_chain_id: int,
#     to_chain_id: int,
#     amount: int,
#     near_client: NearClient,
#     near_contract_id: str,
#     evm_factory_provider,
#     agent_address: str,
#     max_bridge_fee: int,
#     min_finality_threshold: int,
# ):
#     print(f"🟨 Flow Aave→Aave | from={from_chain_id} to={to_chain_id} amount={amount}")
    # await step_aave_withdraw(
    #     from_chain_id=from_chain_id, amount=amount,
    #     near_client=near_client, near_contract_id=near_contract_id, evm_factory_provider=evm_factory_provider
    # )
    # burn_hash = await step_cctp_burn(
    #     from_chain_id=from_chain_id, to_chain_id=to_chain_id, amount=amount,
    #     agent_address=agent_address, max_bridge_fee=max_bridge_fee, min_finality_threshold=min_finality_threshold,
    #     near_client=near_client, near_contract_id=near_contract_id, evm_factory_provider=evm_factory_provider
    # )
    # att = await _wait_attestation(burn_hash, from_chain_id, to_chain_id, min_finality_threshold)
    # await step_cctp_mint(
    #     to_chain_id=to_chain_id, attestation_payload=att,
    #     near_client=near_client, near_contract_id=near_contract_id, evm_factory_provider=evm_factory_provider
    # )
    # await step_aave_supply(
    #     to_chain_id=to_chain_id, amount=amount,
    #     near_client=near_client, near_contract_id=near_contract_id, evm_factory_provider=evm_factory_provider
    # )
    # print(" ✅ Flow Aave→Aave completed.\n")

class AaveToRebalancer(Strategy):
    def __init__(self, *, rebalancer_contract: RebalancerContract) -> None:
        self.rebalancer_contract = rebalancer_contract

    async def execute(self, *, from_chain_id: int, to_chain_id: int, amount: int) -> None:
        print(f"🟩 Flow Aave→Rebalancer | from={from_chain_id} to={to_chain_id} amount={amount}")
        nonce = await self.rebalancer_contract.start_rebalance(flow=Flow.AaveToRebalancer, source_chain=from_chain_id, destination_chain=to_chain_id, expected_amount=amount)
        withdraw_payload = await self.rebalancer_contract.build_aave_withdraw_tx(from_chain_id=from_chain_id, amount=amount)
        # await tx_propagator.send_raw_tx(withdraw_payload)
        # burn_payload = await self.rebalancer_contract.build_cctp_burn_tx(from_chain_id=from_chain_id, to_chain_id=to_chain_id, amount=amount, nonce=nonce)
        # att  = await wait_for_attestation(burn_tx_hash=burn, from_chain_id=from_chain_id,
        #                            to_chain_id=to_chain_id, min_finality_threshold=self.rebalancer_contract.min_finality)
        # mint_payload = await self.rebalancer_contract.build_cctp_mint_tx(to_chain_id=to_chain_id, attestation_payload=att)
        # deposit_payload = await self.rebalancer_contract.build_rebalancer_deposit_tx(to_chain_id=to_chain_id, amount=amount)
        print("✅ Done Aave→Rebalancer\n")