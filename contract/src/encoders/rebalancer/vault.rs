use alloy_primitives::U256;
use alloy_sol_types::{sol, SolCall};

sol! {
    function harvest(uint256 _yieldAmount);
    function invest(uint256 _amount);
}

pub fn encode_harvest(yield_amount: u128) -> Vec<u8> {
    harvestCall {
        _yieldAmount: U256::from(yield_amount),
    }
    .abi_encode()
}

pub fn encode_invest(amount: u128) -> Vec<u8> {
    investCall {
        _amount: U256::from(amount),
    }
    .abi_encode()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_harvest() {
        let data = encode_harvest(1_000_000);
        assert!(data.len() > 4);
    }

    #[test]
    fn test_encode_invest() {
        let data = encode_invest(2_500_000);
        assert!(data.len() > 4);
    }
}
