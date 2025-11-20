from web3 import Web3
from statistics import median

from near_omni_client.providers.evm.alchemy_provider import AlchemyFactoryProvider
from near_omni_client.networks.network import Network

class GasEstimator:
    def __init__(self, evm_factory_provider: AlchemyFactoryProvider):
        self.evm_factory_provider = evm_factory_provider

    def estimate_gas_limit(
        self,
        network: Network,
        from_address: str,
        to_address: str,
        data: bytes,
        value: int = 0,
        buffer: float = 1.2
    ) -> int:
        """
        Estimate gas limit for a transaction by simulating it.
        Adds a safety buffer (default 20%).
        """
        print(f"data (hex): {data.hex()}")
        web3 = self.evm_factory_provider.get_provider(network=network)
        if not web3:
            raise ValueError("Web3 provider is not initialized.")

        tx = {
            "from": Web3.to_checksum_address(from_address),
            "to": Web3.to_checksum_address(to_address) if to_address else None,
            "data": Web3.to_hex(data),
            "value": value,
        }
        print(f"Estimating gas for tx: {tx}")
        try:
            estimate = web3.eth.estimate_gas(tx)
        except Exception as e:
            # fallback to a safe default if estimation fails
            print(f"Gas estimation failed: {e}, using default 500,000")
            return 500000

        # add buffer and return as int
        return int(estimate * buffer)
    
    def get_eip1559_fees(self, network: Network) -> dict:
        """
        Get EIP-1559 fees for the next block on the specified EVM network.
        This method retrieves the base fee and priority fee for the next block,
        and calculates the max fee per gas with a safety margin.
        """
        web3 = self.evm_factory_provider.get_provider(network=network)
        if not web3:
            raise ValueError("Web3 provider is not initialized.")
       
        history = web3.eth.fee_history(5, "latest", reward_percentiles=[50])
        base_fee = history["baseFeePerGas"][-1]

        # Priority fee: tries to use eth_maxPriorityFeePerGas, if not, uses median from history
        try:
            priority = web3.eth.max_priority_fee
        except Exception:
            rewards = [r[0] for r in history["reward"] if r]
            priority = int(median(rewards)) if rewards else web3.to_wei(2, "gwei")

        # Clamp priority fee to a range between 1 gwei and 5 gwei
        priority = max(web3.to_wei(1, "gwei"), min(priority, web3.to_wei(5, "gwei")))

        # Max fee with a safety margin x2 to avoid being left out if baseFee rises
        max_fee = base_fee * 2 + priority

        return {
            "base_fee_per_gas": int(base_fee),
            "max_priority_fee_per_gas": int(priority),
            "max_fee_per_gas": int(max_fee),
        }