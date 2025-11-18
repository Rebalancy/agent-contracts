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


class Assert:
    rebalancer_vault_address: str | None = None
    agent_address: str | None = None

    @classmethod
    def configure(cls, *, rebalancer_vault_address: str, agent_address: str):
        cls.rebalancer_vault_address = rebalancer_vault_address
        cls.agent_address = agent_address

    @classmethod
    def _ensure_config(cls):
        if cls.rebalancer_vault_address is None or cls.agent_address is None:
            raise RuntimeError("Assert not configured. Call Assert.configure() first.")

    #
    # Vault
    #

    @classmethod
    def usdc_vault_balance(cls, web3_instance: Web3, usdc_address: str, expected_balance):
        cls._ensure_config()

        vault_balance = (
            web3_instance.eth.contract(address=usdc_address, abi=USDC_ABI)
            .functions.balanceOf(cls.rebalancer_vault_address)
            .call()
        )

        assert vault_balance == expected_balance, (
            f"Vault USDC balance mismatch! "
            f"Expected: {expected_balance}, Found: {vault_balance}"
        )

    @classmethod
    def atoken_vault_balance(cls, web3_instance: Web3, atoken_address: str, expected_balance):
        cls._ensure_config()

        vault_balance = (
            web3_instance.eth.contract(address=atoken_address, abi=ATOKEN_ABI)
            .functions.balanceOf(cls.rebalancer_vault_address)
            .call()
        )

        assert vault_balance == expected_balance, (
            f"Aave Vault aUSDC balance mismatch! "
            f"Expected: {expected_balance}, Found: {vault_balance}"
        )

    #
    # Agent
    #

    @classmethod
    def usdc_agent_balance(cls, web3_instance: Web3, usdc_address: str, expected_balance):
        cls._ensure_config()

        agent_balance = (
            web3_instance.eth.contract(address=usdc_address, abi=USDC_ABI)
            .functions.balanceOf(cls.agent_address)
            .call()
        )

        assert agent_balance == expected_balance, (
            f"Agent USDC balance mismatch! "
            f"Expected: {expected_balance}, Found: {agent_balance}"
        )

    @classmethod
    def atoken_agent_balance(cls, web3_instance: Web3, atoken_address: str, expected_balance):
        cls._ensure_config()

        agent_balance = (
            web3_instance.eth.contract(address=atoken_address, abi=ATOKEN_ABI)
            .functions.balanceOf(cls.agent_address)
            .call()
        )

        assert agent_balance == expected_balance, (
            f"Agent aUSDC balance mismatch! "
            f"Expected: {expected_balance}, Found: {agent_balance}"
        )