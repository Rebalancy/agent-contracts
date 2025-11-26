from typing import Dict, List

from agent.src.steps.tx_types import infer_flow
from .strategy_manager import StrategyManager

async def execute_all_rebalance_operations(
    source_chain_id: int,
    rebalance_operations: List[Dict[str, int]]):
    for op in rebalance_operations:
        from_chain_id = op["from"]
        to_chain_id = op["to"]

        flow = infer_flow(from_chain_id=from_chain_id, to_chain_id=to_chain_id, source_chain_id=source_chain_id)
        await StrategyManager.get_strategy(flow).execute(from_chain_id=from_chain_id, to_chain_id=to_chain_id, amount=op["amount"])