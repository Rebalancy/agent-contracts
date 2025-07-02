const USDC_DECIMALS: u32 = 6;

pub fn to_usdc_units(amount: f64) -> u64 {
    let factor = 10u64.pow(USDC_DECIMALS);
    (amount * factor as f64).round() as u64
}
