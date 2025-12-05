import asyncio
import base64
import ast

from typing import Any, Dict

from near_omni_client.json_rpc.client import NearClient
from near_omni_client.wallets.near_wallet import NearWallet
from near_omni_client.transactions import TransactionBuilder, ActionFactory
from near_omni_client.transactions.utils import decode_key
from near_omni_client.providers.interfaces import IProviderFactory
from near_omni_client.json_rpc.exceptions import JsonRpcError

from config import Config
from helpers.evm_transaction import create_partial_tx
from helpers import GasEstimator
from engine_types import Flow

from utils import address_to_bytes32, extract_signed_rlp_without_prefix, from_chain_id_to_network, hex_to_int_list, parse_chain_configs, parse_u32_result, parse_chain_balances, extract_signed_rlp, parse_supported_chains

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

    async def get_supported_chains(self) -> list[int]:
        supported_chains_raw = await self.near_client.call_contract(
            contract_id=self.near_contract_id,
            method="get_supported_chains",
            args={}
        )
        return parse_supported_chains(supported_chains_raw)

    async def start_rebalance(self, flow: Flow, source_chain: int, destination_chain: int, expected_amount: int) -> int:        
        args = {
            "flow": flow.name,
            "source_chain": source_chain,
            "destination_chain": destination_chain,
            "amount": expected_amount,
        }

        result = await self._sign_and_submit_transaction(
            method="start_rebalance",
            args=args,
            gas=self.config.tx_tgas * TGAS,
            deposit=0
        )

        success_value_b64 = result.status.get("SuccessValue")
        if not success_value_b64:
            raise Exception("start_rebalance didn't return SuccessValue")

        nonce = int(base64.b64decode(success_value_b64).decode())
        print(f"✅ nonce = {nonce}")
        
        return nonce
    
    async def complete_rebalance(self) -> int:        
        args = {}

        result = await self._sign_and_submit_transaction(
            method="complete_rebalance",
            args=args,
            gas=self.config.tx_tgas * TGAS,
            deposit=0
        )

        success_value_b64 = result.status.get("SuccessValue")
        if not success_value_b64:
            raise Exception("complete_rebalance didn't return SuccessValue")

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

        response = await self.near_client.call_contract(
            contract_id=self.near_contract_id,
            method="build_cctp_approve_before_burn_tx",
            args=args
        )
        print("Created cctp_approve_before_burn payload")
        raw = response.result
        as_str = bytes(raw).decode("utf-8")
        int_list = ast.literal_eval(as_str)
        payload_bytes = bytes(int_list)
        
        return payload_bytes

    async def build_and_sign_cctp_approve_before_burn_tx(self, source_chain: int, amount: int, spender: str,to: str):
        source_chain_as_network = from_chain_id_to_network(source_chain)
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

        success_value_b64 = result.status.get("SuccessValue")
        if not success_value_b64:
            raise Exception("withdraw_for_crosschain_allocation didn't return SuccessValue")

        signed_rlp = extract_signed_rlp(success_value_b64)
                
        return signed_rlp

    async def build_cctp_burn_tx(self, destination_domain: int, amount: int, max_fee: int, burn_token: str):
        print(f"Building cctp_burn tx")        
        args = {
            "amount": amount,
            "destination_domain": destination_domain,
            "mint_recipient": "0x" + self.agent_address_as_bytes32.hex(),
            "burn_token": burn_token,
            "destination_caller": "0x" + self.agent_address_as_bytes32.hex(),
            "max_fee": max_fee,
            "min_finality_threshold": self.config.min_bridge_finality_threshold
        }

        response = await self.near_client.call_contract(
            contract_id=self.near_contract_id,
            method="build_cctp_burn_tx",
            args=args
        )
        print("Created cctp_burn payload")
        raw = response.result
        as_str = bytes(raw).decode("utf-8")
        int_list = ast.literal_eval(as_str)
        payload_bytes = bytes(int_list)
        
        return payload_bytes

    async def build_and_sign_cctp_burn_tx(self, source_chain: int, to_chain_id: int, amount: int, max_fee: int, burn_token: str, to: str):
        source_chain_as_network = from_chain_id_to_network(source_chain)
        destination_domain = int(from_chain_id_to_network(to_chain_id).domain)
        input_payload = await self.build_cctp_burn_tx(destination_domain=destination_domain, amount=amount, max_fee=max_fee, burn_token=burn_token)
        gas_limit = self.gas_estimator.estimate_gas_limit(source_chain_as_network, self.agent_address, to, input_payload)
        print(f"Estimated gas limit for burn transaction: {gas_limit}")

        args = {
            "args": {
                "amount": amount,
                "destination_domain": destination_domain,
                "mint_recipient": "0x" + self.agent_address_as_bytes32.hex(),
                "burn_token": burn_token,
                "destination_caller": "0x" + self.agent_address_as_bytes32.hex(),
                "max_fee": max_fee,
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

        success_value_b64 = result.status.get("SuccessValue")
        if not success_value_b64:
            raise Exception("withdraw_for_crosschain_allocation didn't return SuccessValue")

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

        success_value_b64 = result.status.get("SuccessValue")
        if not success_value_b64:
            raise Exception("build_and_sign_cctp_mint_tx didn't return SuccessValue")

        signed_rlp = extract_signed_rlp(success_value_b64)
                
        return signed_rlp

    async def build_aave_supply_tx(self, asset: str, amount: int, on_behalf_of: str, referral_code: int):
        print(f"Building aave_deposit tx")
        args = {
            "asset": asset,
            "amount": amount,
            "on_behalf_of": on_behalf_of,
            "referral_code": referral_code
        }

        response = await self.near_client.call_contract(
            contract_id=self.near_contract_id,
            method="build_aave_supply_tx",
            args=args
        )
        raw = response.result
        as_str = bytes(raw).decode("utf-8")
        int_list = ast.literal_eval(as_str)
        payload_bytes = bytes(int_list)
        return payload_bytes
    
    async def build_and_sign_aave_supply_tx(self, to_chain_id: int, asset: str, amount: int, on_behalf_of: str,referral_code: int, to:str):
        destination_chain_as_network = from_chain_id_to_network(to_chain_id)
        input_payload = await self.build_aave_supply_tx(asset, amount, on_behalf_of, referral_code)
        gas_limit = self.gas_estimator.estimate_gas_limit(destination_chain_as_network, self.agent_address, to, input_payload)
        print(f"Estimated gas limit for supply aave transaction: {gas_limit}")

        args = {
            "args": {
                "amount": amount,
                "partial_transaction": create_partial_tx(destination_chain_as_network, self.agent_address, self.evm_provider, self.gas_estimator, gas_limit).to_dict()
            },
            "callback_gas_tgas": self.config.callback_gas_tgas
        }
        
        result = await self._sign_and_submit_transaction(
            method="build_and_sign_aave_supply_tx", 
            args=args,
            gas=self.config.tx_tgas * TGAS,
            deposit=0
        )

        success_value_b64 = result.status.get("SuccessValue")
        if not success_value_b64:
            raise Exception("build_and_sign_aave_deposit_tx didn't return SuccessValue")

        signed_rlp = extract_signed_rlp(success_value_b64)
                
        return signed_rlp

    async def build_aave_approve_before_supply_tx(self, amount: int, spender: str):
        print(f"Building aave_approve_before_supply tx")
        args = {
            "amount": amount,
            "spender": spender,
        }

        response = await self.near_client.call_contract(
            contract_id=self.near_contract_id,
            method="build_aave_approve_before_supply_tx",
            args=args
        )
        print("Created aave_approve_before_supply payload")
        raw = response.result
        as_str = bytes(raw).decode("utf-8")
        int_list = ast.literal_eval(as_str)
        payload_bytes = bytes(int_list)
        
        return payload_bytes

    async def build_and_sign_aave_approve_before_supply_tx(self, to_chain_id: int, amount: int, spender: str,to: str):
        chain_as_network = from_chain_id_to_network(to_chain_id)
        input_payload = await self.build_aave_approve_before_supply_tx(amount=amount, spender=spender)
        gas_limit = self.gas_estimator.estimate_gas_limit(chain_as_network, self.agent_address, to, input_payload)
        print(f"Estimated gas limit: {gas_limit}")
        
        args = {
            "args": {
                "amount": amount,
                "spender": spender,
                "partial_transaction": create_partial_tx(chain_as_network, self.agent_address, self.evm_provider, self.gas_estimator, gas_limit).to_dict()
            },
            "callback_gas_tgas": self.config.callback_gas_tgas
        }
        
        result = await self._sign_and_submit_transaction(
            method="build_and_sign_aave_approve_before_supply_tx",
            args=args,
            gas=self.config.tx_tgas * TGAS,
            deposit=0
        )
        print(f"Received result from build_and_sign_aave_approve_before_supply_tx call {result}")

        success_value_b64 = result.status.get("SuccessValue")
        if not success_value_b64:
            raise Exception("aave_approve_before_supply didn't return SuccessValue")

        signed_rlp = extract_signed_rlp(success_value_b64)
                
        return signed_rlp
    
    async def build_aave_withdraw_tx(self, asset: str, amount: int, on_behalf_of: str):
        print(f"Building aave_withdraw tx")
        args = {
            "asset": asset,
            "amount": amount,
            "on_behalf_of": on_behalf_of
        }

        response = await self.near_client.call_contract(
            contract_id=self.near_contract_id,
            method="build_aave_withdraw_tx",
            args=args
        )
        raw = response.result
        as_str = bytes(raw).decode("utf-8")
        int_list = ast.literal_eval(as_str)
        payload_bytes = bytes(int_list)
        return payload_bytes
    
    async def build_and_sign_aave_withdraw_tx(self, chain_id: int, asset: str, amount: int, on_behalf_of: str, to: str):
        chain_network = from_chain_id_to_network(chain_id)
        input_payload = await self.build_aave_withdraw_tx(asset, amount, on_behalf_of)
        gas_limit = self.gas_estimator.estimate_gas_limit(chain_network, self.agent_address, to, input_payload)
        print(f"Estimated gas limit for withdraw aave transaction: {gas_limit}")

        args = {
            "args": {
                "amount": amount,
                "partial_transaction": create_partial_tx(chain_network, self.agent_address, self.evm_provider, self.gas_estimator, gas_limit).to_dict()
            },
            "callback_gas_tgas": self.config.callback_gas_tgas
        }
        
        result = await self._sign_and_submit_transaction(
            method="build_and_sign_aave_withdraw_tx", 
            args=args,
            gas=self.config.tx_tgas * TGAS,
            deposit=0
        )
        print(f"Received result from build_and_sign_aave_withdraw_tx call {result}")

        success_value_b64 = result.status.get("SuccessValue")
        if not success_value_b64:
            raise Exception("build_and_sign_aave_withdraw_tx didn't return SuccessValue")

        signed_rlp = extract_signed_rlp(success_value_b64)
                
        return signed_rlp

    async def build_approve_vault_to_manage_agents_usdc_tx(self, spender: str):
        print(f"Building approve_vault_to_manage_agents_usdc tx")

        args = {
            "spender": spender,
        }
        
        response = await self.near_client.call_contract(
            contract_id=self.near_contract_id,
            method="build_approve_vault_to_manage_agents_usdc",
            args=args
        )
        raw = response.result
        as_str = bytes(raw).decode("utf-8")
        int_list = ast.literal_eval(as_str)
        payload_bytes = bytes(int_list)
        return payload_bytes
    
    async def build_and_sign_approve_vault_to_manage_agents_usdc_tx(self, to_chain_id: int, spender: str, to: str):
        chain_as_network = from_chain_id_to_network(to_chain_id)
        input_payload = await self.build_approve_vault_to_manage_agents_usdc_tx(spender=spender)
        gas_limit = self.gas_estimator.estimate_gas_limit(chain_as_network, self.agent_address, to, input_payload)
        print(f"Estimated gas limit: {gas_limit}")
        
        args = {
            "partial_transaction": create_partial_tx(chain_as_network, self.agent_address, self.evm_provider, self.gas_estimator, gas_limit).to_dict(),
            "callback_gas_tgas": self.config.callback_gas_tgas
        }
        
        result = await self._sign_and_submit_transaction(
            method="build_and_sign_approve_vault_to_manage_agents_usdc_tx",
            args=args,
            gas=self.config.tx_tgas * TGAS,
            deposit=0
        )
        print(f"Received result from build_and_sign_approve_vault_to_manage_agents_usdc_tx call {result}")
        
        success_value_b64 = result.status.get("SuccessValue")
        if not success_value_b64:
            raise Exception("approve_vault_to_manage_agents_usdc didn't return SuccessValue")

        signed_rlp = extract_signed_rlp_without_prefix(success_value_b64)
                
        return signed_rlp

    async def build_return_funds_tx(self, amount: int, cross_chain_a_token_balance: int):
        print(f"Building return_funds tx")
        args = {
            "amount": amount,
            "cross_chain_a_token_balance": cross_chain_a_token_balance
        }

        response = await self.near_client.call_contract(
            contract_id=self.near_contract_id,
            method="build_return_funds_tx",
            args=args
        )
        raw = response.result
        as_str = bytes(raw).decode("utf-8")
        int_list = ast.literal_eval(as_str)
        payload_bytes = bytes(int_list)
        return payload_bytes

    async def build_and_sign_return_funds_tx(self, to_chain_id: int, amount: int, cross_chain_a_token_balance: int, to: str):
        chain_as_network = from_chain_id_to_network(to_chain_id)
        input_payload = await self.build_return_funds_tx(amount=amount, cross_chain_a_token_balance=cross_chain_a_token_balance)
        gas_limit = self.gas_estimator.estimate_gas_limit(chain_as_network, self.agent_address, to, input_payload)
        print(f"Estimated gas limit: {gas_limit}")
        
        args = {
            "args": {
                "amount": amount,
                "cross_chain_a_token_balance": cross_chain_a_token_balance,
                "partial_transaction": create_partial_tx(chain_as_network, self.agent_address, self.evm_provider, self.gas_estimator, gas_limit).to_dict()
            },
            "callback_gas_tgas": self.config.callback_gas_tgas
        }
        
        result = await self._sign_and_submit_transaction(
            method="build_and_sign_return_funds_tx",
            args=args,
            gas=self.config.tx_tgas * TGAS,
            deposit=0
        )

        success_value_b64 = result.status.get("SuccessValue")
        if not success_value_b64:
            raise Exception("return_funds didn't return SuccessValue")

        signed_rlp = extract_signed_rlp_without_prefix(success_value_b64)
                
        return signed_rlp

    async def _sign_and_submit_transaction(self, *, method: str, args: Dict[str, Any], gas: int, deposit: int, max_retries: int = 3, delay: float = 2.0):
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
        
        # --- Retry section starts here ---
        for attempt in range(1, max_retries + 1):
            try:
                print(f"Sending transaction to NEAR network... (attempt {attempt})")
                result = await self.near_client.send_raw_transaction(signed_tx_base64)
                print("✅ Transaction successfully sent.")
                return result
            except JsonRpcError as e:
                if "TIMEOUT_ERROR" in str(e):
                    if attempt < max_retries:
                        print(f"⚠️  Timeout error on attempt {attempt}. Retrying in {delay:.1f}s...")
                        await asyncio.sleep(delay)
                        delay *= 2  # exponential backoff
                        continue
                    else:
                        print("❌ Transaction failed after maximum retries due to TIMEOUT_ERROR.")
                raise
            except Exception as e:
                # Catch unexpected errors (network, aiohttp, etc.)
                if attempt < max_retries:
                    print(f"⚠️  Unexpected error on attempt {attempt}: {e}. Retrying in {delay:.1f}s...")
                    await asyncio.sleep(delay)
                    delay *= 2
                    continue
                else:
                    print("❌ Transaction failed after maximum retries due to unexpected error.")
                    raise
        # --- Retry section ends here ---
        
