from typing import Dict, List

from .flows import execute_rebalancer_to_aave_flow, execute_aave_to_rebalancer_flow, execute_aave_to_aave_flow
from .types import Flow, infer_flow

async def execute_all_rebalance_operations(
    source_chain_id: int,
    rebalance_operations: List[Dict[str, int]],
    near_client,
    evm_factory_provider,
    near_wallet,
    near_contract_id: str,
    agent_address: str,
    max_bridge_fee: float,
    min_finality_threshold: int,
    gas_for_rebalancer: int = 10,
    gas_for_cctp_burn: int = 10,
):
    for op in rebalance_operations:
        from_chain_id = op["from"]
        to_chain_id = op["to"]

        flow = infer_flow(from_chain_id=from_chain_id, to_chain_id=to_chain_id, source_chain_id=source_chain_id)

        if flow == Flow.RebalancerToAave:
            print("🟦 Executing Flow: Rebalancer → Aave")
            await execute_rebalancer_to_aave_flow(
                from_chain_id=from_chain_id,
                to_chain_id=to_chain_id,
                amount=op["amount"],
                near_client=near_client,
                evm_factory_provider=evm_factory_provider,
                near_wallet=near_wallet,
                near_contract_id=near_contract_id,
                agent_address=agent_address,
                max_bridge_fee=max_bridge_fee,
                min_finality_threshold=min_finality_threshold,
                gas_for_rebalancer=gas_for_rebalancer,
                gas_for_cctp_burn=gas_for_cctp_burn,
            )
        elif flow == Flow.AaveToRebalancer:
            print("🟩 Executing Flow: Aave → Rebalancer")
            await execute_aave_to_rebalancer_flow(
                from_chain_id=from_chain_id,
                to_chain_id=to_chain_id,
                amount=op["amount"],
                near_client=near_client,
                evm_factory_provider=evm_factory_provider,
                near_wallet=near_wallet,
                near_contract_id=near_contract_id,
                agent_address=agent_address,
                max_bridge_fee=max_bridge_fee,
                min_finality_threshold=min_finality_threshold,
                gas_for_rebalancer=gas_for_rebalancer,
                gas_for_cctp_burn=gas_for_cctp_burn,
            )
        elif flow == Flow.AaveToAave:
            print("🟨 Executing Flow: Aave → Aave")
            await execute_aave_to_aave_flow(
                from_chain_id=from_chain_id,
                to_chain_id=to_chain_id,
                amount=op["amount"],
                near_client=near_client,
                evm_factory_provider=evm_factory_provider,
                near_wallet=near_wallet,
                near_contract_id=near_contract_id,
                agent_address=agent_address,
                max_bridge_fee=max_bridge_fee,
                min_finality_threshold=min_finality_threshold,
                gas_for_cctp_burn=gas_for_cctp_burn,
            )
        else:
            raise ValueError(f"Unknown flow type: {flow}")    
   