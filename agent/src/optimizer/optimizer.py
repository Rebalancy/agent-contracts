import numpy as np
from scipy.optimize import minimize

def optimize_chain_allocation_with_direction(data):
    """
    Optimizes allocation of funds across multiple AAVE chains with direction-aware elasticity.
    
    Args:
        data (dict): Dictionary containing:
            - totalAssetsUnderManagement: Total funds to allocate
            - chains: List of chain data, each with:
                - chainId: Chain id
                - currentAllocation: Current funds allocated
                - currentInterestRate: Current liquidity interest rate (%)
                - supplyElasticity: How much interest rate changes per 1% utilization
                - totalSupply: Total supplied assets in the market
                - totalBorrow: Total borrowed assets in the market
    
    Returns:
        dict: Optimized allocations across chains
    """
    # Extract inputs
    total_funds = data["totalAssetsUnderManagement"]
    chains = data["chains"]
    
    # Check if we have chains to optimize
    if not chains:
        return data
    
    # Extract chain data into arrays for optimization
    n_chains = len(chains)
    chain_ids = [chain["chainId"] for chain in chains]
    current_rates = np.array([chain["currentInterestRate"] for chain in chains])
    supply_elasticity = np.array([chain["supplyElasticity"] for chain in chains])
    total_supply = np.array([chain["totalSupply"] for chain in chains])
    total_borrow = np.array([chain["totalBorrow"] for chain in chains])
    current_alloc = np.array([chain["currentAllocation"] for chain in chains])
    
    # Calculate current utilization rates
    current_utilization = total_borrow / total_supply * 100  # percentage
    
    # Initial allocation proportions (current state)
    x0 = current_alloc / total_funds
    
    # Objective function to maximize (negative for minimization)
    def objective(x):
        # Calculate the net change in allocation for each chain
        new_allocation = x * total_funds
        allocation_change = new_allocation - current_alloc
        
        # Calculate new total supply for each chain after reallocation
        new_total_supply = total_supply + allocation_change
        
        # Calculate new utilization rates
        new_utilization = np.where(
            new_total_supply > 0,  # Check to avoid division by zero
            total_borrow / new_total_supply * 100,
            0
        )
        
        # Calculate utilization change (in percentage points)
        utilization_change = new_utilization - current_utilization
        
        # Calculate new interest rates based on direction-aware elasticity
        # When utilization increases, rates increase; when utilization decreases, rates decrease
        new_rates = current_rates + supply_elasticity * utilization_change
        
        # Ensure rates don't go negative
        new_rates = np.maximum(new_rates, 0)
        
        # Weighted average return (negative for minimization)
        return -np.sum(x * new_rates)
    
    # Constraints: sum of allocations = 1
    constraints = [{'type': 'eq', 'fun': lambda x: np.sum(x) - 1.0}]
    
    # Bounds: allocations between 0 and 1
    bounds = [(0, 1) for _ in range(n_chains)]
    
    # Solve optimization problem
    result = minimize(objective, x0, method='SLSQP', bounds=bounds, constraints=constraints)
    
    if result.success:
        # Convert optimal proportions to allocations
        optimal_alloc = result.x * total_funds

        # 1) Current average interest rate
        #    Weighted by the current allocations
        current_alloc_proportions = current_alloc / total_funds
        current_average_interest = np.sum(current_alloc_proportions * current_rates)

        # 2) Updated interest rate after rebalancing
        delta_u = (optimal_alloc - current_alloc) / total_supply * 100
        updated_rates = current_rates + supply_elasticity * delta_u

        # 3) Projected average interest rate
        #    Weighted by the new allocation proportions (result.x)
        projected_average_interest = np.sum(result.x * updated_rates)

        # 4) Projected change in interest rates for each chain
        projected_interest_changes = updated_rates - current_rates

        # Prepare output with new allocations
        new_allocations = {}
        for i, chain_id in enumerate(chain_ids):
            ## TRY COMMENTING OUT THE RESTRICTION ON OPTIMAL ALLOCATION
            # # Return null if allocation didn't change significantly (within 1%)
            # if abs(optimal_alloc[i] - current_alloc[i]) < 0.05 * current_alloc[i]:
            #     new_allocations[chain_id] = None
            # else:
                new_allocations[chain_id] = round(optimal_alloc[i])

        return {
            "totalAssetsUnderManagement": total_funds,
            "allocations": new_allocations,
            "currentAverageInterestRate": f"{current_average_interest:.2f}%",
            "updatedInterestRates": [f"{rate:.2f}%" for rate in updated_rates],
            "projectedInterestRateChanges": [f"{change:.2f}%" for change in projected_interest_changes],
            "projectedAverageInterestRate": f"{projected_average_interest:.2f}%"
        }
    else:
        return {
            "totalAssetsUnderManagement": total_funds,
            "allocations": {chain_id: round(current_alloc[i]) for i, chain_id in enumerate(chain_ids)}
        }