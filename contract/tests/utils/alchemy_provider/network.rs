use std::str::FromStr;

/// Enum de redes compatibles con Alchemy (slugs oficiales).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Network {
    BaseSepolia,
    BaseMainnet,
    EthereumSepolia,
    EthereumMainnet,
    ArbitrumSepolia,
    ArbitrumMainnet,
    OptimismSepolia,
    OptimismMainnet,
}

impl Network {
    /// Slug de Alchemy para la red (parte que va en https://{slug}.g.alchemy.com/v2/{api_key})
    pub fn alchemy_slug(&self) -> &'static str {
        match self {
            Network::BaseSepolia => "base-sepolia",
            Network::BaseMainnet => "base-mainnet",
            Network::EthereumSepolia => "eth-sepolia",
            Network::EthereumMainnet => "eth-mainnet",
            Network::ArbitrumSepolia => "arb-sepolia",
            Network::ArbitrumMainnet => "arb-mainnet",
            Network::OptimismSepolia => "opt-sepolia",
            Network::OptimismMainnet => "opt-mainnet",
        }
    }
}

impl FromStr for Network {
    type Err = ();

    /// Permite parsear desde strings tipo "base-sepolia", "eth-mainnet", etc.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "base-sepolia" => Network::BaseSepolia,
            "base-mainnet" => Network::BaseMainnet,
            "eth-sepolia" => Network::EthereumSepolia,
            "eth-mainnet" => Network::EthereumMainnet,
            "arb-sepolia" => Network::ArbitrumSepolia,
            "arb-mainnet" => Network::ArbitrumMainnet,
            "opt-sepolia" => Network::OptimismSepolia,
            "opt-mainnet" => Network::OptimismMainnet,
            _ => return Err(()),
        })
    }
}
