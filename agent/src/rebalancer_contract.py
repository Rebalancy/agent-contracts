import base64
import ast

from typing import Any, Dict

from near_omni_client.json_rpc.client import NearClient
from near_omni_client.wallets.near_wallet import NearWallet
from near_omni_client.transactions import TransactionBuilder, ActionFactory
from near_omni_client.transactions.utils import decode_key
from near_omni_client.providers.interfaces import IProviderFactory

from evm_transaction import create_partial_tx
from tx_types import Flow
from config import Config
from gas_estimator import GasEstimator

from utils import address_to_bytes32, from_chain_id_to_network, hex_to_int_list, parse_chain_configs, parse_u32_result, parse_chain_balances, extract_signed_rlp

TGAS = 1_000_000_000_000  # 1 TeraGas

class RebalancerContract:
    def __init__(self, near_client: NearClient, near_wallet: NearWallet, near_contract_id: str, agent_address: str, gas_estimator: GasEstimator, evm_provider: IProviderFactory, config: Config) -> None:
        self.near_client = near_client
        self.near_contract_id = near_contract_id
        self.near_wallet = near_wallet
        self.agent_address = agent_address
        self.gas_estimator = gas_estimator
        self.evm_provider = evm_provider
        self.config = config
        self.agent_address_as_bytes32 = address_to_bytes32(self.agent_address)

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
            gas=300_000_000_000_000, # TODO: Make it constant or from ENV Vars
            deposit=0
        )
        print("result", result)

        success_value_b64 = result.status.get("SuccessValue")
        if not success_value_b64:
            raise Exception("start_rebalance didn't return SuccessValue")

        nonce = int(base64.b64decode(success_value_b64).decode())
        print(f"✅ nonce = {nonce}")
        
        return nonce
        
    async def build_withdraw_for_crosschain_allocation_tx(self, amount: int, cross_chain_a_token_balance: Any = None):
        print(f"Building withdraw_for_crosschain_allocation tx")
        args = {
            "amount": amount,
            "cross_chain_a_token_balance": cross_chain_a_token_balance
        }

        response = await self.near_client.call_contract(
            contract_id=self.near_contract_id,
            method="build_withdraw_for_crosschain_allocation_tx",
            args=args
        )
        raw = response.result
        as_str = bytes(raw).decode("utf-8")
        int_list = ast.literal_eval(as_str)
        payload_bytes = bytes(int_list)
        return payload_bytes

    async def build_and_sign_withdraw_for_crosschain_allocation_tx(self, source_chain: int, amount: int, to: str):
        source_chain_as_network = from_chain_id_to_network(source_chain)
        input_payload = await self.build_withdraw_for_crosschain_allocation_tx(amount=amount)
        gas_limit = self.gas_estimator.estimate_gas_limit(source_chain_as_network, self.agent_address, to, input_payload)
        
        args = {
            "rebalancer_args": {
                "amount": amount,
                "partial_transaction": create_partial_tx(source_chain_as_network, self.agent_address, self.evm_provider, self.gas_estimator, gas_limit).to_dict(),
                "cross_chain_a_token_balance": None
            },
            "callback_gas_tgas": self.config.callback_gas_tgas
        }

        result = await self._sign_and_submit_transaction(
            method="build_and_sign_withdraw_for_crosschain_allocation_tx",
            args=args,
            gas=self.config.tx_tgas * TGAS,
            deposit=0
        )

        success_value_b64 = result.status.get("SuccessValue")
        if not success_value_b64:
            raise Exception("withdraw_for_crosschain_allocation didn't return SuccessValue")

        signed_rlp = extract_signed_rlp(success_value_b64)
                
        return signed_rlp

    async def build_cctp_approve_before_burn_tx(self, amount: int, spender: str):
        print(f"Building cctp_approve_before_burn tx")
        args = {
            "amount": amount,
            "spender": spender,
        }
        print("ARGS build_cctp_approve_before_burn_tx:", args)
        print("ARGS TYPES:", {k: type(v) for k, v in args.items()})

        response = await self.near_client.call_contract(
            contract_id=self.near_contract_id,
            method="build_cctp_approve_before_burn_tx",
            args=args
        )
        print("Created cctp_approve_before_burn payload")
        raw = response.result
        print("raw response", raw)
        as_str = bytes(raw).decode("utf-8")
        int_list = ast.literal_eval(as_str)
        payload_bytes = bytes(int_list)
        
        return payload_bytes

    async def build_and_sign_cctp_approve_before_burn_tx(self, source_chain: int, to_chain_id: int, amount: int, spender: str,to: str):
        source_chain_as_network = from_chain_id_to_network(source_chain)
        destination_domain = int(from_chain_id_to_network(to_chain_id).domain)
        input_payload = await self.build_cctp_approve_before_burn_tx(amount=amount, spender=spender)
        gas_limit = self.gas_estimator.estimate_gas_limit(source_chain_as_network, self.agent_address, to, input_payload)
        print(f"Estimated gas limit: {gas_limit}")
        
        args = {
            "args": {
                "amount": amount,
                "spender": spender,
                "partial_transaction": create_partial_tx(source_chain_as_network, self.agent_address, self.evm_provider, self.gas_estimator, gas_limit).to_dict()
            },
            "callback_gas_tgas": self.config.callback_gas_tgas
        }
        
        result = await self._sign_and_submit_transaction(
            method="build_and_sign_cctp_approve_before_burn_tx",
            args=args,
            gas=self.config.tx_tgas * TGAS,
            deposit=0
        )
        print("result", result)

        success_value_b64 = result.status.get("SuccessValue")
        if not success_value_b64:
            raise Exception("withdraw_for_crosschain_allocation didn't return SuccessValue")

        print("success_value_b64", success_value_b64)
        signed_rlp = extract_signed_rlp(success_value_b64)
                
        return signed_rlp

    async def build_cctp_burn_tx(self, destination_domain: int, amount: int, burn_token: str):
        print(f"Building cctp_burn tx")        
        args = {
            "amount": amount,
            "destination_domain": destination_domain,
            "mint_recipient": "0x" + self.agent_address_as_bytes32.hex(),
            "burn_token": burn_token,
            "destination_caller": "0x" + self.agent_address_as_bytes32.hex(),
            "max_fee": self.config.max_bridge_fee,
            "min_finality_threshold": self.config.min_bridge_finality_threshold
        }
        print("ARGS build_cctp_burn_tx:", args)
        print("ARGS TYPES:", {k: type(v) for k, v in args.items()})

        response = await self.near_client.call_contract(
            contract_id=self.near_contract_id,
            method="build_cctp_burn_tx",
            args=args
        )
        print("Created cctp_burn payload")
        raw = response.result
        print("raw response", raw)
        as_str = bytes(raw).decode("utf-8")
        int_list = ast.literal_eval(as_str)
        payload_bytes = bytes(int_list)
        
        return payload_bytes

    async def build_and_sign_cctp_burn_tx(self, source_chain: int, to_chain_id: int, amount: int, burn_token: str, to: str):
        source_chain_as_network = from_chain_id_to_network(source_chain)
        destination_domain = int(from_chain_id_to_network(to_chain_id).domain)
        input_payload = await self.build_cctp_burn_tx(destination_domain=destination_domain, amount=amount, burn_token=burn_token)
        gas_limit = self.gas_estimator.estimate_gas_limit(source_chain_as_network, self.agent_address, to, input_payload)
        print(f"Estimated gas limit for burn transaction: {gas_limit}")

        args = {
            "args": {
                "amount": amount,
                "destination_domain": destination_domain,
                "mint_recipient": "0x" + self.agent_address_as_bytes32.hex(),
                "burn_token": burn_token,
                "destination_caller": "0x" + self.agent_address_as_bytes32.hex(),
                "max_fee": self.config.max_bridge_fee,
                "min_finality_threshold": self.config.min_bridge_finality_threshold,
                "partial_burn_transaction": create_partial_tx(source_chain_as_network, self.agent_address, self.evm_provider, self.gas_estimator, gas_limit).to_dict()
            },
            "callback_gas_tgas": self.config.callback_gas_tgas
        }
        
        result = await self._sign_and_submit_transaction(
            method="build_and_sign_cctp_burn_tx",
            args=args,
            gas=self.config.tx_tgas * TGAS,
            deposit=0
        )
        print("result", result)

        success_value_b64 = result.status.get("SuccessValue")
        if not success_value_b64:
            raise Exception("withdraw_for_crosschain_allocation didn't return SuccessValue")

        print("success_value_b64", success_value_b64)
        signed_rlp = extract_signed_rlp(success_value_b64)
                
        return signed_rlp
    
    async def build_cctp_mint_tx(self, message: str, attestation: str):
        print(f"Building cctp_mint tx")
        args = {
            "message": hex_to_int_list(message),
            "attestation": hex_to_int_list(attestation)
        }

        response = await self.near_client.call_contract(
            contract_id=self.near_contract_id,
            method="build_cctp_mint_tx",
            args=args
        )
        print("Created cctp_mint payload")
        print("raw response", response)
        raw = response.result
        as_str = bytes(raw).decode("utf-8")
        int_list = ast.literal_eval(as_str)
        payload_bytes = bytes(int_list)
        return payload_bytes
    
    async def build_and_sign_cctp_mint_tx(self, to_chain_id: int, message: str, attestation: str, to: str): 
        print(f"Building and signing cctp_mint tx")
        print(f"chain id: {to_chain_id}")
        destination_chain_as_network = from_chain_id_to_network(to_chain_id)
        input_payload = await self.build_cctp_mint_tx(message, attestation)
        gas_limit = self.gas_estimator.estimate_gas_limit(destination_chain_as_network, self.agent_address, to, input_payload)
        print(f"Estimated gas limit: {gas_limit}")
       
        args = {
            "args": {
                "message": hex_to_int_list(message),
                "attestation": hex_to_int_list(attestation),
                "partial_mint_transaction": create_partial_tx(destination_chain_as_network, self.agent_address, self.evm_provider, self.gas_estimator, gas_limit=gas_limit).to_dict(), 
            },
            "callback_gas_tgas": self.config.callback_gas_tgas
        }
        
        result = await self._sign_and_submit_transaction(
            method="build_and_sign_cctp_mint_tx", 
            args=args,
            gas=self.config.tx_tgas * TGAS,
            deposit=0
        )
        print("result", result)

        success_value_b64 = result.status.get("SuccessValue")
        if not success_value_b64:
            raise Exception("build_and_sign_cctp_mint_tx didn't return SuccessValue")

        print("success_value_b64", success_value_b64)
        signed_rlp = extract_signed_rlp(success_value_b64)
                
        return signed_rlp

    async def build_aave_deposit_tx(self, to_chain_id: int, amount: int):
        print(f"Building aave_deposit tx")
        args = {
            "amount": amount,
            "partial_deposit_transaction": {}
        }

        response = await self.near_client.call_contract(
            contract_id=self.near_contract_id,
            method="build_aave_deposit_tx",
            args=args
        )
        raw = response.result
        as_str = bytes(raw).decode("utf-8")
        int_list = ast.literal_eval(as_str)
        payload_bytes = bytes(int_list)
        return payload_bytes
    
    async def build_and_sign_aave_deposit_tx(self, to_chain_id: int, amount: int):
        destination_chain_as_network = from_chain_id_to_network(to_chain_id)
        
        args = {
            "amount": amount,
            "partial_deposit_transaction": create_partial_tx(destination_chain_as_network, self.agent_address, self.evm_provider, self.gas_estimator).to_dict(),
        }
        
        result = await self._sign_and_submit_transaction(
            method="build_and_sign_aave_deposit_tx", 
            args=args,
            gas=self.config.tx_tgas * TGAS,
            deposit=0
        )
        print("result", result)

        success_value_b64 = result.status.get("SuccessValue")
        if not success_value_b64:
            raise Exception("build_and_sign_aave_deposit_tx didn't return SuccessValue")

        print("success_value_b64", success_value_b64)
        signed_rlp = extract_signed_rlp(success_value_b64)
                
        return signed_rlp

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
    
