
# Mapping of known explorers for direct transaction links
EXPLORERS = {
    1: "https://etherscan.io/tx/",
    5: "https://goerli.etherscan.io/tx/",
    11155111: "https://sepolia.etherscan.io/tx/",
    42161: "https://arbiscan.io/tx/",
    421614: "https://sepolia.arbiscan.io/tx/",
    137: "https://polygonscan.com/tx/",
    80002: "https://amoy.polygonscan.com/tx/",
    10: "https://optimistic.etherscan.io/tx/",
    11155420: "https://sepolia-optimism.etherscan.io/tx/",
}

def broadcast(web3, payload: bytes) -> str:
    """
    Broadcasts a raw transaction through the given Web3 provider.

    This function:
      - Verifies the provider is initialized
      - Sends the raw transaction
      - Detects the chain_id automatically
      - Prints a link to the corresponding block explorer (if available)
      - Raises exceptions on failure (for state-machine error propagation)

    Returns:
        str: The transaction hash (hex)
    
    Raises:
        ValueError: If provider is not initialized
        RuntimeError: If broadcasting fails
    """
    if not web3:
        raise ValueError("Web3 provider not initialized.")

    try:
        # Send the transaction
        tx_hash = web3.eth.send_raw_transaction(payload)
        hex_hash = tx_hash.hex()

        # Detect chain and build explorer link
        chain_id = getattr(web3.eth, "chain_id", None)
        explorer_base = EXPLORERS.get(chain_id)
        explorer_link = f"{explorer_base}0x{hex_hash}" if explorer_base else None

        # Log result
        if explorer_link:
            print(f"✅ Transaction broadcasted successfully:\n🔗 {explorer_link}")
        else:
            print(f"✅ Transaction broadcasted: 0x{hex_hash}")
            print(f"ℹ️ No explorer configured for chain_id {chain_id}")

        return hex_hash

    except Exception as e:
        raise RuntimeError(f"Error broadcasting transaction: {e}") from e