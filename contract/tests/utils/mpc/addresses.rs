use k256::elliptic_curve::sec1::FromEncodedPoint;
use k256::elliptic_curve::sec1::ToEncodedPoint;
use k256::EncodedPoint;
use k256::{
    elliptic_curve::{bigint::ArrayEncoding, CurveArithmetic, PrimeField},
    AffinePoint, Scalar, Secp256k1, U256,
};
use near_sdk::AccountId;
use sha3::Sha3_256;
use tiny_keccak::{Hasher, Keccak};

pub type PublicKey = <Secp256k1 as CurveArithmetic>::AffinePoint;

trait ScalarExt: Sized {
    fn from_bytes(bytes: [u8; 32]) -> Option<Self>;
    fn from_non_biased(bytes: [u8; 32]) -> Self;
}

impl ScalarExt for Scalar {
    /// Returns nothing if the bytes are greater than the field size of Secp256k1.
    /// This will be very rare with random bytes as the field size is 2^256 - 2^32 - 2^9 - 2^8 - 2^7 - 2^6 - 2^4 - 1
    fn from_bytes(bytes: [u8; 32]) -> Option<Self> {
        let bytes = U256::from_be_slice(bytes.as_slice());
        Self::from_repr(bytes.to_be_byte_array()).into_option()
    }

    /// When the user can't directly select the value, this will always work
    /// Use cases are things that we know have been hashed
    fn from_non_biased(hash: [u8; 32]) -> Self {
        // This should never happen.
        // The space of inputs is 2^256, the space of the field is ~2^256 - 2^129.
        // This mean that you'd have to run 2^127 hashes to find a value that causes this to fail.
        Self::from_bytes(hash).expect("Derived epsilon value falls outside of the field")
    }
}

// Constant prefix that ensures epsilon derivation values are used specifically for
// near-mpc-recovery with key derivation protocol vX.Y.Z.
const EPSILON_DERIVATION_PREFIX: &str = "near-mpc-recovery v0.1.0 epsilon derivation:";

/// Derives an epsilon value from a given predecessor_id and path
pub fn derive_epsilon(predecessor_id: &AccountId, path: &str) -> Scalar {
    let derivation_path = format!("{EPSILON_DERIVATION_PREFIX}{},{}", predecessor_id, path);
    let mut hasher = Sha3_256::new();
    hasher.update(derivation_path);
    let hash: [u8; 32] = hasher.finalize().into();
    Scalar::from_non_biased(hash)
}

/// Derives a key from a given public key and epsilon value
pub fn derive_key(public_key: PublicKey, epsilon: Scalar) -> PublicKey {
    (<Secp256k1 as CurveArithmetic>::ProjectivePoint::GENERATOR * epsilon + public_key).to_affine()
}

const ROOT_PUBLIC_KEY: &str = "secp256k1:4NfTiv3UsGahebgTaHyD9vF8KYKMBnfd6kh94mK6xv8fGBiJB8TBtFMP5WWXz6B89Ac1fbpzPwAvoyQebemHFwx3";

/// Contains the derived address as string and the public key
/// that was used to derive the address
pub struct DerivedAddress {
    pub address: String,
    pub public_key: PublicKey,
}

/// Derives an EVM address for a given path and predecessor_id
///
/// Example:
/// ```
/// let derived_address = get_derived_address_for_evm("omnitester.testnet".parse().unwrap(), "ethereum-1")
/// ```
pub fn get_derived_address_for_evm(predecessor_id: &AccountId, path: &str) -> DerivedAddress {
    let epsilon = derive_epsilon(predecessor_id, path);
    let public_key = convert_string_to_public_key(ROOT_PUBLIC_KEY).unwrap();
    let derived_public_key = derive_key(public_key, epsilon);
    let evm_address = public_key_to_evm_address(derived_public_key);

    DerivedAddress {
        address: evm_address,
        public_key: derived_public_key,
    }
}

/// Converts a string-encoded public key to a public key (AffinePoint) non compressed
fn convert_string_to_public_key(encoded: &str) -> Result<PublicKey, String> {
    let base58_part = encoded.strip_prefix("secp256k1:").ok_or("Invalid prefix")?;

    let mut decoded_bytes = bs58::decode(base58_part)
        .into_vec()
        .map_err(|_| "Base58 decoding failed")?;

    if decoded_bytes.len() != 64 {
        return Err(format!(
            "Invalid public key length: expected 64, got {}",
            decoded_bytes.len()
        ));
    }

    decoded_bytes.insert(0, 0x04);

    let public_key = EncodedPoint::from_bytes(&decoded_bytes).unwrap();

    let public_key = AffinePoint::from_encoded_point(&public_key).unwrap();

    Ok(public_key)
}

/// Converts a public key to an Ethereum address
pub fn public_key_to_evm_address(public_key: AffinePoint) -> String {
    let encoded_point = public_key.to_encoded_point(false);
    let public_key_bytes = encoded_point.as_bytes();

    // Exclude the first byte (0x04 prefix) and take only the X part
    let x_only = &public_key_bytes[1..];

    // Calculate the hash Keccak-256
    let mut keccak = Keccak::v256();
    let mut hash = [0u8; 32];
    keccak.update(x_only);
    keccak.finalize(&mut hash);

    // Take the last 20 bytes
    let eth_address_bytes = &hash[12..];

    format!("0x{}", hex::encode(eth_address_bytes))
}

#[allow(dead_code)]
fn public_key_to_hex(public_key: AffinePoint) -> String {
    let encoded_point = public_key.to_encoded_point(false);
    let encoded_point_bytes = encoded_point.as_bytes();

    hex::encode(encoded_point_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_epsilon() {
        let predecessor_id = "omnitester.testnet".parse().unwrap();
        let path = "bitcoin-1";

        let epsilon = derive_epsilon(&predecessor_id, path);

        let public_key = convert_string_to_public_key("secp256k1:4NfTiv3UsGahebgTaHyD9vF8KYKMBnfd6kh94mK6xv8fGBiJB8TBtFMP5WWXz6B89Ac1fbpzPwAvoyQebemHFwx3").unwrap();

        let derived_public_key = derive_key(public_key, epsilon);

        let derived_public_key_hex = public_key_to_hex(derived_public_key);

        assert_eq!(derived_public_key_hex, "0471f75dc56b971fbe52dd3e80d2f8532eb8905157556df39cb7338a67c80412640c869f717217ba5b916db6d7dc7d6a84220f8251e626adad62cac9c7d6f8e032");
    }

    #[test]
    fn test_evm_address() {
        let predecessor_id = "omnitester.testnet".parse().unwrap();
        let path = "ethereum-1";

        let epsilon = derive_epsilon(&predecessor_id, path);

        let public_key = convert_string_to_public_key("secp256k1:4NfTiv3UsGahebgTaHyD9vF8KYKMBnfd6kh94mK6xv8fGBiJB8TBtFMP5WWXz6B89Ac1fbpzPwAvoyQebemHFwx3").unwrap();

        let derived_public_key = derive_key(public_key, epsilon);

        let derived_public_key_hex = public_key_to_hex(derived_public_key);

        let evm_address = public_key_to_evm_address(derived_public_key);

        assert_eq!(evm_address, "0xd8d25820c9b9e2aa9cce55504355e500efcce715");
        assert_eq!(derived_public_key_hex, "04e612e7650febebc50b448bf790f6bdd70a8a6ce3b111a1d7e72c87afe84be776e36226e3f89de1ba3cbb62c0f3fc05bffae672c9c59d5fa8a4737b6547c64eb7");
    }
}
