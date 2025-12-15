use crate::{Contract, ContractExt};
use near_sdk::{near, require};

#[near]
impl Contract {
    pub fn complete_rebalance(&mut self) -> u64 {
        self.assert_agent_is_calling();

        let session = self.get_active_session();
        let nonce = session.nonce;

        require!(
            self.active_session.is_some(),
            "No active session to complete"
        );

        self.active_session = None;

        nonce
    }
}

#[cfg(test)]
mod maintests {
    use crate::test_helpers::*;
    use crate::types::*;
    use near_sdk::env;

    #[test]
    fn test_() {}
}
