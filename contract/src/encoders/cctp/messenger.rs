use alloy_primitives::{Address, B256, U256};
use alloy_sol_types::{sol, SolCall};

sol! {
    function depositForBurn(
        uint256 amount,
        uint32 destinationDomain,
        bytes32 mintRecipient,
        address burnToken,
        bytes32 destinationCaller,
        uint256 maxFee,
        uint32 minFinalityThreshold
    );
}

pub fn encode_deposit_for_burn(
    amount: U256,
    destination_domain: u32,
    mint_recipient: B256,
    burn_token: Address,
    destination_caller: B256,
    max_fee: U256,
    min_finality_threshold: u32,
) -> Vec<u8> {
    depositForBurnCall {
        amount,
        destinationDomain: destination_domain,
        mintRecipient: mint_recipient,
        burnToken: burn_token,
        destinationCaller: destination_caller,
        maxFee: max_fee,
        minFinalityThreshold: min_finality_threshold,
    }
    .abi_encode()
}

#[cfg(test)]
mod tests {
    use super::super::messenger::encode_deposit_for_burn;
    use alloy_primitives::{B256, U256};

    #[test]
    fn test_encode_deposit_for_burn() {
        let amount = U256::from(1_000_000u64);
        let destination_domain = 0u32;
        let mint_recipient = B256::ZERO;
        let burn_token = "0x".parse().unwrap();
        let destination_caller = B256::ZERO;
        let max_fee = U256::from(10_000u64);
        let min_finality_threshold = 1u32;

        let data = encode_deposit_for_burn(
            amount,
            destination_domain,
            mint_recipient,
            burn_token,
            destination_caller,
            max_fee,
            min_finality_threshold,
        );

        // Opcional: compare the output with expected values
        assert!(data.len() > 4); // Ensure that the data is not empty
    }
}
