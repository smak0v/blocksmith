use chrono::Utc;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::info;

use crate::chain::DIFFICULTY_PREFIX;
use crate::utils;

#[derive(Debug, Clone, Getters, Serialize, Deserialize)]
pub struct Block {
    id: u64,
    hash: String,
    prev_hash: String,
    timestamp: i64,
    data: String,
    nonce: u64,
}

impl Block {
    pub fn new(id: u64, prev_hash: String, data: String) -> Self {
        let now = Utc::now();
        let (nonce, hash) = Block::mine(id, now.timestamp(), &prev_hash, &data);

        Self {
            id,
            hash,
            prev_hash,
            timestamp: now.timestamp(),
            data,
            nonce,
        }
    }

    fn mine(id: u64, timestamp: i64, previous_hash: &str, data: &str) -> (u64, String) {
        info!("mining block...");

        let mut nonce = 0;

        loop {
            if nonce % 100000 == 0 {
                info!("nonce: {}", nonce);
            }

            let hash = calculate_hash(id, timestamp, previous_hash, data, nonce);
            let binary_hash = utils::hex_to_binary(&hash);

            if binary_hash.starts_with(DIFFICULTY_PREFIX) {
                info!(
                    "mined! nonce: {}, hash: {}, binary hash: {}",
                    nonce,
                    hex::encode(&hash),
                    binary_hash
                );

                return (nonce, hex::encode(hash));
            }

            nonce += 1;
        }
    }
}

pub fn calculate_hash(id: u64, timestamp: i64, prev_hash: &str, data: &str, nonce: u64) -> Vec<u8> {
    let data = serde_json::json!({
        "id": id,
        "prev_hash": prev_hash,
        "data": data,
        "timestamp": timestamp,
        "nonce": nonce
    });
    let mut hasher = Sha256::new();

    hasher.update(data.to_string().as_bytes());

    hasher.finalize().to_vec()
}
