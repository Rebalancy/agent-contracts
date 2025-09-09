from typing import Dict, List

from .types import infer_flow
from .strategy_manager import StrategyManager

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
        # TODO: Revisar que se pasan bien los parametros en cada llamada y que realmente es lo que se necesita pasar....
        await StrategyManager.get_strategy(flow).execute(from_chain_id=from_chain_id, to_chain_id=to_chain_id, amount=op["amount"])