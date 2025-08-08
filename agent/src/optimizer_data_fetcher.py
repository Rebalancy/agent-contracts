from web3 import Web3
from near_omni_client.adapters.aave import LendingPool
from near_omni_client.wallets import MPCWallet

from utils import from_chain_id_to_network

async def get_extra_data_for_optimization(total_assets_under_management, mpc_wallet: MPCWallet, current_allocations, configs, override_interest_rates: dict = None) -> dict:
    
    data = {
        "chains": [],
        "totalAssetsUnderManagement": total_assets_under_management
    }

    for chain_id, allocation in current_allocations.items():
        network = from_chain_id_to_network(chain_id)
        asset_in_network = configs[chain_id]["aave"]["asset"]
        asset_address = Web3.to_checksum_address(asset_in_network)
        lending_pool = LendingPool(wallet=mpc_wallet, network=network)

        if override_interest_rates and chain_id in override_interest_rates:
            interest_rate = override_interest_rates[chain_id]
        else:
            interest_rate =  lending_pool.get_interest_rate(asset_address)
            
        slope =  lending_pool.get_slope(asset_address)
        total_supply, total_borrow =  lending_pool.get_supply_and_borrow(asset_address)
        
        data["chains"].append({
            "chainId": chain_id,
            "currentAllocation": allocation,
            "currentInterestRate": interest_rate,
            "supplyElasticity": slope,
            "totalSupply": total_supply,
            "totalBorrow": total_borrow
        })

    return data


    