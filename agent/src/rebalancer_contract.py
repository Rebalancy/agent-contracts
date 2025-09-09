import base64
import json
from typing import Any, Dict

from near_omni_client.json_rpc.client import NearClient
from near_omni_client.wallets.near_wallet import NearWallet
from near_omni_client.transactions import TransactionBuilder, ActionFactory
from near_omni_client.transactions.utils import decode_key
from near_omni_client.providers.interfaces import IProviderFactory
from .evm_transaction import create_partial_tx
from .types import Flow
from .utils import from_chain_id_to_network, parse_chain_configs, parse_u32_result, parse_chain_balances
from .gas_estimator import GasEstimator

class RebalancerContract:
    def __init__(self, near_client: NearClient, near_wallet: NearWallet, near_contract_id: str, agent_address: str, gas_estimator: GasEstimator, evm_provider: IProviderFactory) -> None:
        self.near_client = near_client
        self.near_contract_id = near_contract_id
        self.near_wallet = near_wallet
        self.agent_address = agent_address
        self.gas_estimator = gas_estimator
        self.evm_provider = evm_provider

    async def get_all_configs(self):
        chain_config_raw = await self.near_client.call_contract(
            contract_id=self.near_contract_id,
            method="get_all_configs",
            args={}
        )
        return parse_chain_configs(chain_config_raw)

    async def get_source_chain(self):
        source_chain_raw = await self.near_client.call_contract(
            contract_id=self.near_contract_id,
            method="get_source_chain",
            args={}
        )
        return parse_u32_result(source_chain_raw)

    async def get_allocations(self):
        allocations_raw = await self.near_client.call_contract(
            contract_id=self.near_contract_id,
            method="get_allocations",
            args={}
        )
        return parse_chain_balances(allocations_raw)

    async def start_rebalance(self, flow: Flow, source_chain: int, destination_chain: int, expected_amount: int) -> int:        
        args = {
            "flow": flow.name,
            "source_chain": source_chain,
            "destination_chain": destination_chain,
            "expected_amount": expected_amount
        }

        result = await self._sign_and_submit_transaction(
            method="start_rebalance",
            args=args,
            gas=300_000_000_000_000,
            deposit=0
        )
        print("result", result)

        success_value_b64 = result.status.get("SuccessValue")
        if not success_value_b64:
            raise Exception("start_rebalance didn't return SuccessValue")

        nonce = int(base64.b64decode(success_value_b64).decode())
        print(f"✅ nonce = {nonce}")
        
        return nonce
        

    async def build_withdraw_for_crosschain_allocation_tx(self, nonce: int, source_chain: int, destination_chain: int, amount: int):
        source_chain_as_network = from_chain_id_to_network(source_chain)
        args = {
            "rebalancer_args": {
                "amount": 1,
                "partial_transaction": create_partial_tx(source_chain_as_network, self.agent_address, self.evm_provider, self.gas_estimator),
                "cross_chain_a_token_balance": None
            },
            "callback_gas_tgas": destination_chain
        }
        result = await self._sign_and_submit_transaction(
            method="withdraw_for_crosschain_allocation",
            args=args,
            gas=300_000_000_000_000,
            deposit=0
        )

        print("result", result)

        success_value_b64 = result.status.get("SuccessValue")
        if not success_value_b64:
            raise Exception("withdraw_for_crosschain_allocation didn't return SuccessValue")

        nonce = int(base64.b64decode(success_value_b64).decode())
        print(f"✅ nonce = {nonce}")
        
        # call get_signed_transactions
        signed_transactions_result = await self.near_client.call_contract(
            contract_id=self.near_contract_id,
            method="get_signed_transactions",
            args={
                "nonce": nonce
            }
        )

        print("signed_transactions result", signed_transactions_result)

        raw_result = signed_transactions_result.result  # list[int] de ASCII codes
        
        parsed = json.loads(bytes(raw_result).decode("utf-8"))  # ahora sí lista de listas de bytes
        
        signed_transactions_bytes = [bytes(tx) for tx in parsed]
        
        print("✅ signed_transactions_bytes:", signed_transactions_bytes)

    async def _sign_and_submit_transaction(self, *, method: str, args: Dict[str, Any], gas: int, deposit: int):
        public_key_str = await self.near_wallet.get_public_key()
        signer_account_id = self.near_wallet.get_address()
        private_key_str = self.near_wallet.keypair.to_string()
        nonce_and_block_hash = await self.near_client.get_nonce_and_block_hash(signer_account_id, public_key_str)
        
        tx = (
            TransactionBuilder()
            .with_signer_id(signer_account_id)
            .with_public_key(public_key_str)
            .with_nonce(nonce_and_block_hash["nonce"])
            .with_receiver(self.near_contract_id)
            .with_block_hash(nonce_and_block_hash["block_hash"])
            .add_action(
                ActionFactory.function_call(
                    method_name=method,
                    args=args,
                    gas=gas,
                    deposit=deposit,
                )
            )
            .build()
        )

        private_key_bytes = decode_key(private_key_str)
        signed_tx = tx.to_vec(private_key_bytes)
        signed_tx_bytes = bytes(bytearray(signed_tx))
        signed_tx_base64 = base64.b64encode(signed_tx_bytes).decode("utf-8")
        
        print("Sending transaction to NEAR network...")
        result = await self.near_client.send_raw_transaction(signed_tx_base64)
        return result