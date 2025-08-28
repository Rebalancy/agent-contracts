use near_sdk::{env, require};

use crate::Contract;

impl Contract {
    pub(crate) fn assert_agent_is_calling(&self) {
        self.require_worker_has_valid_codehash();
    }

    fn require_worker_has_valid_codehash(&self) {
        // let worker = self.get_worker(env::predecessor_account_id());
        // require!(self.approved_codehashes.contains(&worker.codehash));
        // TODO: Uncomment above after testing
    }

    pub(crate) fn require_owner(&self) {
        require!(env::predecessor_account_id() == self.owner_id);
    }

    pub(crate) fn require_approved_codehash(&self, codehash: &String) {
        require!(self.approved_codehashes.contains(codehash));
    }
}
