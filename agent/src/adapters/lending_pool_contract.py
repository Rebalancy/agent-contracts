from web3 import Web3
from .abis import LENDING_POOL_ABI

class LendingPool:
    @staticmethod
    def get_atoken_address(web3_instance: Web3, lending_pool_address: str, asset_address: str) -> str:
        """Return the aToken address for a given asset."""
        contract = web3_instance.eth.contract(address=lending_pool_address, abi=LENDING_POOL_ABI)

        # getReserveData
        reserve_data = contract.functions.getReserveData(asset_address).call()
        # extract the aToken address from the reserve data
        a_token = reserve_data[8]

        return a_token