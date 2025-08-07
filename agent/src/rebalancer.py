import base64
from typing import Dict, List
from near_omni_client.transactions import ActionFactory, TransactionBuilder
from near_omni_client.transactions.utils import decode_key

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

# async def execute_all_rebalances(
#     current_allocations: Dict[int, int],
#     optimized_allocations: Dict[int, int],
#     near_client,
#     contract_id: str,
#     one_time_signer_account_id: str,
#     one_time_signer_public_key: str,
#     one_time_signer_private_key: str,
#     AGENT_ADDRESS: str,
#     CHAIN_ID_TO_DOMAIN: Dict[int, int],
#     CHAIN_ID_TO_USDC: Dict[int, str],
#     to_usdc_units,
#     MIN_FINALITY_THRESHOLD,
#     execute_rebalance
# ):
#     rebalance_ops = compute_rebalance_operations(current_allocations, optimized_allocations)

#     for op in rebalance_ops:
#         tx = empty_tx_for_chain(op["from"])

#         rebalance_args = {
#             "source_chain": op["from"],
#             "destination_chain": op["to"],
#             "rebalancer_args": {
#                 "amount": op["amount"],
#                 "source_chain": op["from"],
#                 "destination_chain": op["to"],
#                 "partial_transaction": tx
#             },
#             "cctp_args": {
#                 "amount": op["amount"],
#                 "destination_domain": CHAIN_ID_TO_DOMAIN[op["to"]],
#                 "mint_recipient": AGENT_ADDRESS,
#                 "burn_token": CHAIN_ID_TO_USDC[op["from"]],
#                 "destination_caller": AGENT_ADDRESS,
#                 "max_fee": to_usdc_units(0.99),
#                 "min_finality_threshold": MIN_FINALITY_THRESHOLD,
#                 "message": [],
#                 "attestation": [],
#                 "partial_burn_transaction": tx,
#                 "partial_mint_transaction": tx
#             },
#             "gas_for_rebalancer": 10,
#             "gas_for_cctp_burn": 10,
#         }

#         await execute_rebalance(
#             near_client=near_client,
#             signer_account_id=one_time_signer_account_id,
#             public_key_str=one_time_signer_public_key,
#             receiver_account_id=contract_id,
#             private_key_str=one_time_signer_private_key,
#             rebalance_args=rebalance_args
#         )

async def execute_rebalance(near_client, signer_account_id, public_key_str, receiver_account_id, private_key_str, rebalance_args):
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
    result = await near_client.send_raw_transaction(signed_tx_base64)
    print("result", result)

    # nonce = result.get("nonce", None)

    # call get_signed_transactions

    # propagate signatures

    # CCTP Wait For Attestation

    # Complete Rebalance