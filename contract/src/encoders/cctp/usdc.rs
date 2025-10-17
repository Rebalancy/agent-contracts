use alloy_primitives::{Address, U256};
use alloy_sol_types::{sol, SolCall};

sol! {
    function approve(address spender, uint256 amount) returns (bool);
}

pub fn encode_approve(spender: Address, amount: U256) -> Vec<u8> {
    approveCall {
        spender: spender.into(),
        amount: amount.into(),
    }
    .abi_encode()
}

#[cfg(test)]
mod tests {
    use alloy::primitives::U256;

    use super::super::usdc::encode_approve;

    #[test]
    fn test_encode_approve() {
        let spender = "0x7d2768de84f9a91b2c744cf0f0865d2e4b30f4bf"
            .parse()
            .unwrap();
        let amount = U256::from(10_000u64);

        let data = encode_approve(spender, amount);

        assert!(data.len() > 4); // Verify that the data is not empty
    }
}
