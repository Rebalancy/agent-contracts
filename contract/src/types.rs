use near_sdk::near;

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct Worker {
    pub checksum: String,
    pub codehash: String,
}
