import json
from web3 import Web3

from typing import Any, Dict
from near_omni_client.networks import Network

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
    Parsea el resultado de get_all_configs() desde un Vec<u8> con lista de tuplas.

    Args:
        response: La respuesta del contrato con un campo `.result` (list[int])

    Returns:
        Dict[str, dict]: Un dict tipo {chain_id_str: config_dict}
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
    Parsea un `u32` devuelto como string ASCII en un Vec<u8>` de NEAR.

    Ej: result=[56, 52, 53, 51, 50] => 84532

    Args:
        response: objeto con `.result` como list[int]

    Returns:
        int: valor u32 parseado
    """
    if not hasattr(response, "result") or not isinstance(response.result, list):
        raise ValueError("Invalid response format")

    try:
        return int(bytes(response.result).decode("utf-8"))
    except Exception as e:
        raise ValueError(f"Failed to parse u32 from result: {e}")
    
def parse_chain_balances(response: Any) -> Dict[str, int]:
    """
    Parsea un Vec<(ChainId, u128)> donde los u128 vienen como strings.

    Args:
        response: respuesta de `call_contract`, debe tener `.result` como list[int]

    Returns:
        Dict[str, int]: Mapeo de `chain_id` → balance u128 (int en Python)
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