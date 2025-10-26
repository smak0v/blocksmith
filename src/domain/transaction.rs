use derive_getters::Getters;
use secp256k1::{
    Message, PublicKey, Secp256k1, SecretKey,
    ecdsa::Signature,
    hashes::{Hash, sha256::Hash as Sha256Hash},
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use std::str::FromStr;

#[derive(Debug, Clone, Getters, Serialize, Deserialize)]
pub struct Transaction {
    sender: String,
    recipient: String,
    amount: f32,
    data: String,
    signature: String,
}

impl Transaction {
    pub fn new(
        sender: impl Into<String>,
        recipient: impl Into<String>,
        amount: f32,
        data: impl Into<String>,
    ) -> Self {
        Self {
            sender: sender.into(),
            recipient: recipient.into(),
            amount,
            data: data.into(),
            signature: String::new(),
        }
    }

    pub fn sign(&mut self, secret_key: &SecretKey) {
        self.signature = Secp256k1::new()
            .sign_ecdsa(self.create_message(), secret_key)
            .to_string();
    }

    pub fn verify(&self, public_key: &PublicKey) -> bool {
        Secp256k1::new()
            .verify_ecdsa(
                self.create_message(),
                &Signature::from_str(&self.signature).unwrap(),
                public_key,
            )
            .is_ok()
    }

    fn create_message(&self) -> Message {
        let data = json!({
            "sender": self.sender,
            "recipient": self.recipient,
            "amount": self.amount,
            "data": self.data,
        });

        Message::from_digest(Sha256Hash::hash(data.to_string().as_bytes()).to_byte_array())
    }
}
