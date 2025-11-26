from typing import Dict, List

def compute_rebalance_operations(
    current_allocations: Dict[int, int],
    optimized_allocations: Dict[int, int]
) -> List[Dict[str, int]]:
    # Step 1: Calculate the delta for each chain
    delta_by_chain = {
        chain_id: optimized_allocations.get(chain_id, 0) - current_allocations.get(chain_id, 0)
        for chain_id in set(current_allocations.keys()) | set(optimized_allocations.keys())
    }

    # Step 2: Separate chains with surplus (source) and chains with need (destination)
    sources = {cid: delta for cid, delta in delta_by_chain.items() if delta < 0}
    destinations = {cid: delta for cid, delta in delta_by_chain.items() if delta > 0}

    # Step 3: Create sequential rebalance operations
    rebalance_operations = []

    for dst_chain, needed in destinations.items():
        for src_chain, available in list(sources.items()):
            amount = min(-available, needed)
            if amount <= 0:
                continue
            rebalance_operations.append({
                "from": src_chain,
                "to": dst_chain,
                "amount": amount
            })
            sources[src_chain] += amount  # Increase the surplus
            destinations[dst_chain] -= amount  # Decrease the need

            if sources[src_chain] == 0:
                del sources[src_chain]
            if destinations[dst_chain] == 0:
                break

    return rebalance_operations
