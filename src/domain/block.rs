use chrono::Utc;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use tracing::info;

use std::collections::BTreeSet;

use crate::domain::chain::DIFFICULTY_PREFIX;
use crate::domain::transaction::Transaction;
use crate::utils::helpers;

#[derive(Debug, Clone, Getters, Serialize, Deserialize)]
pub struct Block {
    id: u64,
    hash: String,
    prev_hash: String,
    timestamp: i64,
    transactions: BTreeSet<Transaction>,
    nonce: u64,
}

impl Block {
    pub fn new(id: u64, prev_hash: impl Into<String>, transactions: BTreeSet<Transaction>) -> Self {
        let prev_hash = prev_hash.into();
        let (nonce, hash, timestamp) = Block::mine(id, &prev_hash, &transactions);

        Self {
            id,
            hash,
            prev_hash,
            timestamp,
            transactions,
            nonce,
        }
    }

    pub fn calculate_hash(
        id: u64,
        prev_hash: &str,
        timestamp: i64,
        transactions: &BTreeSet<Transaction>,
        nonce: u64,
    ) -> Vec<u8> {
        let data = json!({
            "id": id,
            "prev_hash": prev_hash,
            "timestamp": timestamp,
            "transactions": transactions,
            "nonce": nonce
        });
        let mut hasher = Sha256::new();

        hasher.update(data.to_string().as_bytes());
        hasher.finalize().to_vec()
    }

    fn mine(
        id: u64,
        previous_hash: &str,
        transactions: &BTreeSet<Transaction>,
    ) -> (u64, String, i64) {
        info!("Mining block with id: {}", id);

        let mut nonce = 0;

        loop {
            if nonce % 100_000 == 0 {
                info!("Nonce: {}", nonce);
            }

            let timestamp = Utc::now().timestamp();
            let hex_hash = Block::calculate_hash(id, previous_hash, timestamp, transactions, nonce);
            let binary_hash = helpers::hex_to_binary(&hex_hash);

            if binary_hash.starts_with(DIFFICULTY_PREFIX) {
                let encoded_hex_hash = hex::encode(&hex_hash);

                info!(
                    "Block with id {} is mined with nonce - {}, hash - {}, binary hash - {}",
                    id, nonce, encoded_hex_hash, binary_hash
                );

                return (nonce, encoded_hex_hash, timestamp);
            }

            nonce += 1;
        }
    }
}
