use alloy_primitives::{Address, U256};
use alloy_sol_types::{sol, SolCall};

sol! {
    function supply(address asset, uint256 amount, address onBehalfOf, uint16 referralCode) external;
    function withdraw(address asset, uint256 amount, address to) external returns (uint256);
}

pub fn encode_supply(
    asset: Address,
    amount: U256,
    on_behalf_of: Address,
    referral_code: u16,
) -> Vec<u8> {
    let call = supplyCall {
        asset,
        amount,
        onBehalfOf: on_behalf_of,
        referralCode: referral_code,
    };
    call.abi_encode()
}

pub fn encode_withdraw(asset: Address, amount: U256, to: Address) -> Vec<u8> {
    let call = withdrawCall { asset, amount, to };
    call.abi_encode()
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::hex;

    #[test]
    fn test_supply_encode() {
        let addr = "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
            .parse()
            .unwrap();
        let user = "0x1234567890123456789012345678901234567890"
            .parse()
            .unwrap();
        let amount = U256::from(1_000_000_000_000_000_000u128);
        let result = encode_supply(addr, amount, user, 0);
        println!("supply calldata: 0x{}", hex::encode(result));
    }

    #[test]
    fn test_withdraw_encode() {
        let addr = "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
            .parse()
            .unwrap();
        let user = "0x1234567890123456789012345678901234567890"
            .parse()
            .unwrap();
        let amount = U256::from(500_000_000_000_000_000u128);
        let result = encode_withdraw(addr, amount, user);
        println!("withdraw calldata: 0x{}", hex::encode(result));
    }
}
