use bincode::{Decode, Encode};
use chrono::Utc;
use derive_getters::Getters;
use secp256k1::{
    Message, PublicKey, Secp256k1,
    ecdsa::Signature,
    hashes::{Hash, sha256::Hash as Sha256Hash},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::log::warn;

use std::str::FromStr;

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Getters,
    Serialize,
    Deserialize,
    Encode,
    Decode,
)]
pub struct Transaction {
    from: String,
    to: String,
    amount: u64,
    data: String,
    timestamp: i64,
    nonce: u64,
    signature: String,
}

impl Transaction {
    pub fn new(
        from: impl Into<String>,
        to: impl Into<String>,
        amount: u64,
        data: impl Into<String>,
        nonce: u64,
        signature: impl Into<String>,
    ) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            amount,
            data: data.into(),
            timestamp: Utc::now().timestamp(),
            nonce,
            signature: signature.into(),
        }
    }

    pub fn verify(&self, public_key: &PublicKey) -> bool {
        let signature = match Signature::from_str(&self.signature) {
            Ok(signature) => signature,
            Err(error) => {
                warn!("Invalid signature: {:?}", error);

                return false;
            }
        };

        Secp256k1::new()
            .verify_ecdsa(self.create_message(), &signature, public_key)
            .is_ok()
    }

    pub fn create_message(&self) -> Message {
        let data = json!({
            "from": self.from,
            "to": self.to,
            "amount": self.amount,
            "data": self.data,
            "nonce": self.nonce,
        });

        Message::from_digest(Sha256Hash::hash(data.to_string().as_bytes()).to_byte_array())
    }
}
