from web3 import Web3
from near_omni_client.adapters.aave import LendingPool
from near_omni_client.wallets import MPCWallet

from utils import from_chain_id_to_network
from vault_abi import get_vault_abi

async def get_extra_data_for_optimization(mpc_wallet: MPCWallet, current_allocations, configs, source_chain_id: int) -> dict:
    source_chain_config = configs.get(source_chain_id, None)
    if not source_chain_config:
        raise ValueError(f"Source chain config for chain ID {source_chain_id} not found in configs.")
    print("Source Chain Config:", source_chain_config)

    vault_address = source_chain_config["rebalancer"]["vault_address"]
    if not vault_address:
        raise ValueError(f"Vault address for source chain {source_chain_id} not found in configs.")
    print("Vault Address for Source Chain:", vault_address)

    web3 = mpc_wallet.get_web3(from_chain_id_to_network(source_chain_id))
    contract = web3.eth.contract(address=vault_address, abi=get_vault_abi())
    total_assets_under_management = contract.functions.totalAssets().call()
    print(f"Total Assets Under Management: {total_assets_under_management}")

    data = {
        "chains": [],
        "totalAssetsUnderManagement": total_assets_under_management
    }

    for chain_id, allocation in current_allocations.items():
        network = from_chain_id_to_network(chain_id)
        asset_in_network = configs[chain_id]["aave"]["asset"]
        asset_address = Web3.to_checksum_address(asset_in_network)
        print(f"Chain ID: {chain_id}, Allocation: {allocation}, Asset Address: {asset_address}")
        print("Connecting to network:", network)
        lending_pool = LendingPool(wallet=mpc_wallet, network=network)
        interest_rate =  lending_pool.get_interest_rate(asset_address)
        print(f"Interest Rate for {network}: {interest_rate}")
        slope =  lending_pool.get_slope(asset_address)
        print(f"Slope for {network}: {slope}")
        total_supply, total_borrow =  lending_pool.get_supply_and_borrow(asset_address)

        print(f"Chain ID: {chain_id}, Allocation: {allocation}, Interest Rate: {interest_rate}, Slope: {slope}")
        
        data["chains"].append({
            "chainId": chain_id,
            "currentAllocation": allocation,
            "currentInterestRate": interest_rate,
            "supplyElasticity": slope,
            "totalSupply": total_supply,
            "totalBorrow": total_borrow
        })

    return data


    