import sys
import ast
import base64
import json
from web3 import Web3

from typing import Any, Dict
from near_omni_client.networks import Network
from near_omni_client.chain_signatures.utils import get_evm_address
from near_omni_client.chain_signatures.kdf import Kdf

def parse_chain_config(response: Any) -> dict:
    """
    Extracts and decodes the `result` from a NEAR contract call response.
    
    Args:
        response: The response from `call_contract`, must have `.result` as list[int].
        
    Returns:
        dict: The decoded JSON as a dictionary.
    Raises:
        ValueError: If the response format is invalid or decoding fails.
    """
    if not hasattr(response, "result") or not isinstance(response.result, list):
        raise ValueError("Invalid response format: missing `result` as list[int]")

    try:
        return json.loads(bytes(response.result).decode("utf-8"))
    except Exception as e:
        raise ValueError(f"Failed to decode result: {e}")

def parse_chain_configs(response: Any) -> Dict[str, dict]:
    """
    Parse a NEAR contract response containing chain configurations.
    
    Args:
        response: The response from `call_contract`, must have `.result` as list[int].
    
    Returns:
        Dict[str, dict]: A dictionary mapping chain IDs to their configurations.
    
    Raises:
        ValueError: If the response format is invalid or parsing fails.
    """
    if not hasattr(response, "result") or not isinstance(response.result, list):
        raise ValueError("Invalid response: expected .result to be list[int]")

    try:
        raw = bytes(response.result).decode("utf-8")
        parsed = json.loads(raw)

        if not isinstance(parsed, list):
            raise ValueError("Expected a list of [chain_id, config] pairs")

        return {chain_id: config for chain_id, config in parsed}
    except Exception as e:
        raise ValueError(f"Failed to parse configs: {e}")
    
def parse_u32_result(response) -> int:
    """
    Parse a NEAR contract response that returns a single u32 value.

    Args:
        response: The response from `call_contract`, must have `.result` as list[int].

    Returns:
        int: The u32 value as an integer in Python.

    Raises:
        ValueError: If the response format is invalid or decoding fails.
    """
    if not hasattr(response, "result") or not isinstance(response.result, list):
        raise ValueError("Invalid response format")

    try:
        return int(bytes(response.result).decode("utf-8"))
    except Exception as e:
        raise ValueError(f"Failed to parse u32 from result: {e}")
    
def parse_chain_balances(response: Any) -> Dict[str, int]:
    """
    Parse a Vec<(ChainId, u128)> where the u128 values come as strings.

    Args:
        response: The response from `call_contract`, must have `.result` as list[int]

    Returns:
        Dict[str, int]: Mapping of `chain_id` → balance u128 (int in Python)
    """
    if not hasattr(response, "result") or not isinstance(response.result, list):
        raise ValueError("Invalid response: missing `.result` as list[int]")

    try:
        decoded = bytes(response.result).decode("utf-8")
        parsed = json.loads(decoded)

        if not isinstance(parsed, list):
            raise ValueError("Expected a list of [chain_id, u128_string] pairs")

        return {chain_id: int(balance_str) for chain_id, balance_str in parsed}
    except Exception as e:
        raise ValueError(f"Failed to parse Vec<(ChainId, u128)>: {e}")
    
def to_usdc_units(value: float) -> int:
    return int(value * 1_000_000)  # USDC has 6 decimal places

def from_chain_id_to_network(chain_id: int) -> Network:
    """Convert a chain ID to a Network enum."""
    if chain_id == 84532:
        return Network.BASE_SEPOLIA
    elif chain_id == 8453:
        return Network.BASE_MAINNET
    elif chain_id == 1:
        return Network.ETHEREUM_MAINNET
    elif chain_id == 111155111:
        return Network.ETHEREUM_SEPOLIA
    elif chain_id == 11155420:
        return Network.OPTIMISM_SEPOLIA
    elif chain_id == 10:
        return Network.OPTIMISM_MAINNET
    elif chain_id == 42161:
        return Network.ARBITRUM_MAINNET
    elif chain_id == 421614:
        return Network.ARBITRUM_SEPOLIA
    else:
        raise ValueError(f"Unsupported chain ID: {chain_id}")
    

def address_to_bytes32(addr: str) -> bytes:
    addr = Web3.to_checksum_address(addr)
    addr_bytes = Web3.to_bytes(hexstr=addr)
    # Left-pad with zeros to ensure it is 32 bytes
    return addr_bytes.rjust(32, b'\x00')

def extract_signed_rlp(success_value_b64: str) -> bytes:
    raw = base64.b64decode(success_value_b64)

    int_list = ast.literal_eval(raw.decode("utf-8"))

    payload_bytes = bytes(int_list)

    # Remove the first byte (the 0x01 prefix)
    signed_rlp = payload_bytes[1:]

    return signed_rlp

def calculate_evm_address_for_account_id(account_id: str, network: str, path: str) -> str:
    """
    Calculate the EVM address for a given NEAR account ID.
    
    Args:
        account_id (str): The NEAR account ID.
        network (str): The network short name, e.g., "testnet" or "mainnet".
        path (str): The derivation path.
    """
    agent_public_key = Kdf.derive_public_key(
        root_public_key_str=Kdf.get_root_public_key(network),
        epsilon=Kdf.derive_epsilon(account_id=account_id, path=path)
    )
    
    agent_evm_address = get_evm_address(agent_public_key)

    return agent_evm_address

if __name__ == "__main__":
    if len(sys.argv) != 4:
        print("Usage: python -m agent.src.utils <account_id> <network> <path>")
        sys.exit(1)

    account_id = sys.argv[1]
    network = sys.argv[2]
    path = sys.argv[3]

    evm_addr = calculate_evm_address_for_account_id(account_id, network, path)
    print(evm_addr)