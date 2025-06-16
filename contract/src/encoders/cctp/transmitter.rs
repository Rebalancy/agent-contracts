use alloy_sol_types::{sol, SolCall};

sol! {
    function receiveMessage(bytes message, bytes attestation) returns (bool);
}

pub fn encode_receive_message(message: Vec<u8>, attestation: Vec<u8>) -> Vec<u8> {
    receiveMessageCall {
        message: message.into(),
        attestation: attestation.into(),
    }
    .abi_encode()
}

#[cfg(test)]
mod tests {
    use super::super::transmitter::encode_receive_message;

    #[test]
    fn test_encode_receive_message() {
        let message = b"example message".to_vec();
        let attestation = b"example attestation".to_vec();

        let data = encode_receive_message(message.clone(), attestation.clone());

        assert!(data.len() > 4); // Verify that the data is not empty
    }
}
