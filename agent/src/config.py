import os
import sys
import json

from near_omni_client.networks import Network


class Config:
    """
    Main configuration class for the Rebalance Agent.

    Loads environment variables, applies defaults, parses network enums,
    and validates required fields.

    Typical usage:
        config = Config.from_env()
    """
    def __init__(
        self,
        contract_id: str,
        near_network: Network,
        network_short_name: str,
        alchemy_api_key: str,
        callback_gas_tgas: int,
        tx_tgas: int,
        max_bridge_fee: int,
        min_bridge_finality_threshold: int,
        one_time_signer_private_key: str,
        one_time_signer_account_id: str,
        override_interest_rates: dict[int, float],
    ):
        self.contract_id = contract_id
        self.near_network = near_network
        self.network_short_name = network_short_name
        self.alchemy_api_key = alchemy_api_key
        self.max_bridge_fee = max_bridge_fee
        self.min_bridge_finality_threshold = min_bridge_finality_threshold
        self.one_time_signer_private_key = one_time_signer_private_key
        self.one_time_signer_account_id = one_time_signer_account_id
        self.override_interest_rates = override_interest_rates
        self.callback_gas_tgas = callback_gas_tgas
        self.tx_tgas = tx_tgas

        self._validate()

    @classmethod
    def from_env(cls):
        """
        Create a Config instance by reading environment variables.
        """
        contract_id = os.getenv("NEAR_CONTRACT_ACCOUNT", "rebalancer.testnet")
        near_network_raw = os.getenv("NEAR_NETWORK", "testnet")
        near_network = Network.parse(near_network_raw)
        network_short_name = "testnet" if near_network == Network.NEAR_TESTNET else "mainnet"

        alchemy_api_key = os.getenv("ALCHEMY_API_KEY", "your_alchemy_api_key_here")
        max_bridge_fee = int(os.getenv("MAX_BRIDGE_FEE", "990000"))  # 0.99 USDC (6 decimals)
        min_bridge_finality_threshold = int(os.getenv("MIN_BRIDGE_FINALITY_THRESHOLD", "1000"))
        one_time_signer_private_key = os.getenv("ONE_TIME_SIGNER_PRIVATE_KEY", "your_private_key_here")
        one_time_signer_account_id = os.getenv("ONE_TIME_SIGNER_ACCOUNT_ID", "your_account_id_here")
        callback_gas_tgas = int(os.getenv("CALLBACK_GAS_TGAS", "10"))  # Default to 10 TGas
        tx_tgas = int(os.getenv("TX_TGAS", "300"))  # Default to 300 TGas

        # Parse OVERRIDE_INTEREST_RATES as JSON dict
        override_rates = os.getenv("OVERRIDE_INTEREST_RATES", "{}")
        try:
            override_interest_rates_raw = json.loads(override_rates)
            override_interest_rates = {int(k): v for k, v in override_interest_rates_raw.items()}
        except json.JSONDecodeError:
            raise ValueError("OVERRIDE_INTEREST_RATES must be a valid JSON dictionary.")

        return cls(
            contract_id=contract_id,
            near_network=near_network,
            network_short_name=network_short_name,
            alchemy_api_key=alchemy_api_key,
            max_bridge_fee=max_bridge_fee,
            min_bridge_finality_threshold=min_bridge_finality_threshold,
            one_time_signer_private_key=one_time_signer_private_key,
            one_time_signer_account_id=one_time_signer_account_id,
            override_interest_rates=override_interest_rates,
            callback_gas_tgas=callback_gas_tgas,
            tx_tgas=tx_tgas,
        )

    def _validate(self):
        """
        Validate critical configuration fields.
        Exits if required values are missing.
        """
        if not self.contract_id:
            sys.exit("❌ Missing required env var: NEAR_CONTRACT_ACCOUNT")
        if not self.alchemy_api_key or self.alchemy_api_key == "your_alchemy_api_key_here":
            print("⚠️  Warning: using default ALCHEMY_API_KEY (not production-ready)")
        if not self.one_time_signer_private_key or self.one_time_signer_private_key == "your_private_key_here":
            print("⚠️  Warning: ONE_TIME_SIGNER_PRIVATE_KEY is not set")

    def summary(self):
        """
        Print a readable summary of the current configuration.
        """
        print("-----------------------------------------------------")
        print("Starting Rebalance Agent")
        print(f"Contract ID: {self.contract_id}")
        print(f"Network: {self.near_network} ({self.network_short_name})")
        print(f"Max Bridge Fee: {self.max_bridge_fee}")
        print(f"Min Bridge Finality Threshold: {self.min_bridge_finality_threshold}")
        print(f"One-Time Signer Account ID: {self.one_time_signer_account_id}")
        print(f"Override Interest Rates: {self.override_interest_rates}")
        print(f"Callback Gas (TGas): {self.callback_gas_tgas}")
        print(f"Transaction Gas (TGas): {self.tx_tgas}")
        print("-----------------------------------------------------")