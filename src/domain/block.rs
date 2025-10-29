use bincode::{self, config};
use chrono::Utc;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::info;

use std::collections::BTreeSet;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

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
    pub fn new(
        id: u64,
        prev_hash: impl Into<String>,
        transactions: BTreeSet<Transaction>,
        cancel_mining: Arc<AtomicBool>,
    ) -> Option<Self> {
        let prev_hash = prev_hash.into();

        if let Some((nonce, hash, timestamp)) =
            Block::mine(id, &prev_hash, &transactions, cancel_mining)
        {
            Some(Self {
                id,
                hash,
                prev_hash,
                timestamp,
                transactions,
                nonce,
            })
        } else {
            None
        }
    }

    pub fn calculate_hash(
        id: u64,
        prev_hash: &str,
        timestamp: i64,
        transactions: &BTreeSet<Transaction>,
        nonce: u64,
    ) -> [u8; 32] {
        let mut hasher = Sha256::new();

        hasher.update(id.to_le_bytes());
        hasher.update(prev_hash.as_bytes());
        hasher.update(timestamp.to_le_bytes());

        for transaction in transactions {
            hasher.update(bincode::encode_to_vec(transaction, config::standard()).unwrap());
        }

        hasher.update(nonce.to_le_bytes());
        hasher.finalize().into()
    }

    fn mine(
        id: u64,
        previous_hash: &str,
        transactions: &BTreeSet<Transaction>,
        cancel_mining: Arc<AtomicBool>,
    ) -> Option<(u64, String, i64)> {
        info!("Mining block with id: {}", id);

        let mut nonce = 0;
        let timestamp = Utc::now().timestamp();

        loop {
            if cancel_mining.load(Ordering::Relaxed) {
                println!("Mining cancelled");

                return None;
            }

            if nonce % 100_000 == 0 {
                info!("Nonce: {}", nonce);
            }

            let hex_hash = Block::calculate_hash(id, previous_hash, timestamp, transactions, nonce);
            let binary_hash = helpers::hex_to_binary(&hex_hash);

            if binary_hash.starts_with(DIFFICULTY_PREFIX) {
                let encoded_hex_hash = hex::encode(&hex_hash);

                info!(
                    "Block with id {} is mined with nonce - {}, hash - {}, binary hash - {}",
                    id, nonce, encoded_hex_hash, binary_hash
                );

                return Some((nonce, encoded_hex_hash, timestamp));
            }

            nonce += 1;
        }
    }
}
