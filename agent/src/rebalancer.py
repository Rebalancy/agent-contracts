import base64
from typing import Dict, List

from near_omni_client.transactions import ActionFactory, TransactionBuilder
from near_omni_client.transactions.utils import decode_key
from near_omni_client.adapters.cctp.usdc_contract import USDCContract

from utils import from_chain_id_to_network
from evm_transaction import get_empty_tx_for_chain

def compute_rebalance_operations(
    current_allocations: Dict[int, int],
    optimized_allocations: Dict[int, int]
) -> List[Dict[str, int]]:
    # Calculate the delta for each chain
    delta_by_chain = {
        chain_id: optimized_allocations.get(chain_id, 0) - current_allocations.get(chain_id, 0)
        for chain_id in set(current_allocations.keys()) | set(optimized_allocations.keys())
    }

    # Step 2: Separate chains with surplus (source) and chains with need (destination)
    sources = {cid: delta for cid, delta in delta_by_chain.items() if delta < 0}
    destinations = {cid: delta for cid, delta in delta_by_chain.items() if delta > 0}

    # Step 3: Create sequential rebalance operations
    rebalance_operations = []

    for dst_chain, needed in destinations.items():
        for src_chain, available in list(sources.items()):
            amount = min(-available, needed)
            if amount <= 0:
                continue
            rebalance_operations.append({
                "from": src_chain,
                "to": dst_chain,
                "amount": amount
            })
            sources[src_chain] += amount  # Increase the surplus
            destinations[dst_chain] -= amount  # Decrease the need

            if sources[src_chain] == 0:
                del sources[src_chain]
            if destinations[dst_chain] == 0:
                break

    return rebalance_operations

async def execute_all_rebalances(
    rebalance_operations: List[Dict[str, int]],
    near_client,
    near_wallet,
    near_contract_id: str,
    agent_address: str,
    max_bridge_fee: float,
    min_finality_threshold: int,
    gas_for_rebalancer: int = 10,
    gas_for_cctp_burn: int = 10,
):
    for op in rebalance_operations:
        tx = get_empty_tx_for_chain(op["from"])
        network_for_to_chain = from_chain_id_to_network[op["to"]]
        network_for_from_chain = from_chain_id_to_network[op["from"]]
        usdc_address_on_from_chain = USDCContract.get_address_for_chain(network_for_from_chain)

        rebalance_args = {
            "source_chain": op["from"],
            "destination_chain": op["to"],
            "rebalancer_args": {
                "amount": op["amount"],
                "source_chain": op["from"],
                "destination_chain": op["to"],
                "partial_transaction": tx
            },
            "cctp_args": {
                "amount": op["amount"],
                "destination_domain": network_for_to_chain.domain,
                "mint_recipient": agent_address,
                "burn_token": usdc_address_on_from_chain,
                "destination_caller": agent_address,
                "max_fee": max_bridge_fee,
                "min_finality_threshold": min_finality_threshold,
                "message": [],
                "attestation": [],
                "partial_burn_transaction": tx,
                "partial_mint_transaction": tx
            },
            "gas_for_rebalancer": gas_for_rebalancer,
            "gas_for_cctp_burn": gas_for_cctp_burn,
        }
        print("Rebalance Args:", rebalance_args)
        print("Executing rebalance operation:", op)
        await execute_rebalance(
            near_client=near_client,
            near_wallet=near_wallet,
            receiver_account_id=near_contract_id,
            rebalance_args=rebalance_args
        )

async def execute_rebalance(near_client, near_wallet, receiver_account_id, rebalance_args):
    public_key_str = await near_wallet.get_public_key()
    signer_account_id = near_wallet.get_address()
    private_key_str = near_wallet.keypair.secret_key
    print("signer_account_id", signer_account_id)
    print("public_key_str", public_key_str)
    print("private_key_str", private_key_str)
    nonce_and_block_hash = await near_client.get_nonce_and_block_hash(signer_account_id, public_key_str)

    print("nonce_and_block_hash", nonce_and_block_hash)

    # start_rebalance (build invest rebalancer tx and build cctp burn tx)
    tx_builder = TransactionBuilder()
    tx = (
        tx_builder.with_signer_id(signer_account_id)
        .with_public_key(public_key_str)
        .with_nonce(nonce_and_block_hash["nonce"])
        .with_receiver(receiver_account_id)
        .with_block_hash(nonce_and_block_hash["block_hash"])
        .add_action(
            ActionFactory.function_call(
                method_name="start_rebalance",
                args=rebalance_args,
                gas=300_000_000_000_000,
                deposit=1,
            )
        )
        .build()
    )

    private_key_bytes = decode_key(private_key_str)
    signed_tx = tx.to_vec(private_key_bytes)
    print("signed_tx", signed_tx)

    signed_tx_bytes = bytes(bytearray(signed_tx))
    signed_tx_base64 = base64.b64encode(signed_tx_bytes).decode("utf-8")
    print("signed_tx_base64", signed_tx_base64)

    # 3) Send the transaction
    print("Sending transaction to NEAR network...")
    # result = await near_client.send_raw_transaction(signed_tx_base64)
    # print("result", result)

    # nonce = result.get("nonce", None)

    # call get_signed_transactions

    # propagate signatures

    # CCTP Wait For Attestation

    # Complete Rebalance