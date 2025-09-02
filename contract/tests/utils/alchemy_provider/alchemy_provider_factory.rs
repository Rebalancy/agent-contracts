use alloy::providers::ProviderBuilder;
use url::Url;

use std::collections::HashSet;

use crate::utils::alchemy_provider::{
    iprovider_factory::{HttpProvider, IProviderFactory},
    Network,
};
pub struct AlchemyFactoryProvider {
    api_key: String,
    supported: HashSet<Network>,
}

impl AlchemyFactoryProvider {
    pub fn new<S: Into<String>>(api_key: S) -> Self {
        let supported: HashSet<Network> = [
            Network::BaseSepolia,
            Network::BaseMainnet,
            Network::EthereumSepolia,
            Network::EthereumMainnet,
            Network::ArbitrumSepolia,
            Network::ArbitrumMainnet,
            Network::OptimismSepolia,
            Network::OptimismMainnet,
        ]
        .into_iter()
        .collect();

        Self {
            api_key: api_key.into(),
            supported,
        }
    }

    pub fn alchemy_url_for(&self, network: Network) -> Result<String, String> {
        if !self.supported.contains(&network) {
            return Err(format!("Network {} not supported", network.alchemy_slug()));
        }
        Ok(format!(
            "https://{}.g.alchemy.com/v2/{}",
            network.alchemy_slug(),
            self.api_key
        ))
    }
}

impl IProviderFactory for AlchemyFactoryProvider {
    fn get_provider(&self, network: Network) -> Result<HttpProvider, String> {
        let url_str = self.alchemy_url_for(network)?;
        let url = Url::parse(&url_str).map_err(|e| format!("Invalid URL: {e}"))?;
        let provider: HttpProvider = ProviderBuilder::new().on_http(url); // en tu Alloy no devuelve Result
        Ok(provider)
    }

    fn is_network_supported(&self, network: &Network) -> bool {
        self.supported.contains(network)
    }
}
