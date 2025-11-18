from eth_typing import ChecksumAddress
from web3 import Web3

USDC_ABI = [
    {
        "constant": True,
        "inputs": [{"name": "_owner", "type": "address"}],
        "name": "balanceOf",
        "outputs": [{"name": "balance", "type": "uint256"}],
        "type": "function",
    }
]

ATOKEN_ABI = USDC_ABI


class BalanceHelper:
    rebalancer_vault_address: ChecksumAddress | None = None
    agent_address: ChecksumAddress | None = None

    @classmethod
    def configure(cls, *, rebalancer_vault_address: str, agent_address: str):
        cls.rebalancer_vault_address = Web3.to_checksum_address(rebalancer_vault_address)
        cls.agent_address = Web3.to_checksum_address(agent_address)

    @classmethod
    def _ensure_config(cls):
        if cls.rebalancer_vault_address is None or cls.agent_address is None:
            raise RuntimeError("Assert not configured. Call Assert.configure() first.")

    #
    # Vault
    #

    @classmethod
    def get_usdc_vault_balance(cls, web3_instance: Web3, usdc_address: str):
        cls._ensure_config()

        vault_balance = (
            web3_instance.eth.contract(address=Web3.to_checksum_address(usdc_address), abi=USDC_ABI)
            .functions.balanceOf(cls.rebalancer_vault_address)
            .call()
        )

        return vault_balance

    @classmethod
    def get_atoken_vault_balance(cls, web3_instance: Web3, atoken_address: str):
        cls._ensure_config()

        vault_balance = (
            web3_instance.eth.contract(address=Web3.to_checksum_address(atoken_address), abi=ATOKEN_ABI)
            .functions.balanceOf(cls.rebalancer_vault_address)
            .call()
        )

        return vault_balance

    #
    # Agent
    #

    @classmethod
    def get_usdc_agent_balance(cls, web3_instance: Web3, usdc_address: str):
        cls._ensure_config()

        agent_balance = (
            web3_instance.eth.contract(address=Web3.to_checksum_address(usdc_address), abi=USDC_ABI)
            .functions.balanceOf(cls.agent_address)
            .call()
        )

        return agent_balance

    @classmethod
    def get_atoken_agent_balance(cls, web3_instance: Web3, atoken_address: str):
        cls._ensure_config()

        agent_balance = (
            web3_instance.eth.contract(address=Web3.to_checksum_address(atoken_address), abi=ATOKEN_ABI)
            .functions.balanceOf(cls.agent_address)
            .call()
        )

        return agent_balance