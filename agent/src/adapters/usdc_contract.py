from web3 import Web3
from .abis import USDC_ABI

# TODO: add usdc abi
class USDC:
    @staticmethod
    def get_allowance(web3_instance: Web3, usdc_address: str, spender: str) -> str:
        """Return the aToken address for a given asset."""
        contract = web3_instance.eth.contract(address=usdc_address, abi=USDC_ABI)

        # TODO: Review this
        allowance = contract.functions.allowance(spender).call()       

        return allowance