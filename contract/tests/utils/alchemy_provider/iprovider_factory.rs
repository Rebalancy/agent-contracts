use crate::utils::alchemy_provider::Network;
use alloy::providers::RootProvider;
use alloy::transports::http::{Client, Http};

pub type HttpProvider = RootProvider<Http<Client>>;

/// Interface for a provider factory (minimal, sin thiserror ni enums).
pub trait IProviderFactory {
    /// Returns a provider for the given network or an error message.
    fn get_provider(&self, network: Network) -> Result<HttpProvider, String>;

    /// Returns true if the network is supported.
    fn is_network_supported(&self, network: &Network) -> bool;
}
