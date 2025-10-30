use actix_web::{
    HttpResponse,
    web::{Data, Json},
};
use secp256k1::PublicKey;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use std::str::FromStr;

use crate::api::errors::ApiError;
use crate::api::startup::ApiTxSender;
use crate::domain::transaction::Transaction;

#[derive(Debug, Serialize, Deserialize)]
struct SubmittedTransaction {
    from: String,
    to: String,
    amount: u64,
    data: String,
    nonce: u64,
    signature: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitTransactionRequest {
    transaction: SubmittedTransaction,
    public_key: String,
}

#[tracing::instrument(name = "Submit transaction", skip(api_tx_sender))]
pub async fn submit_transaction(
    params: Json<SubmitTransactionRequest>,
    api_tx_sender: Data<ApiTxSender>,
) -> Result<HttpResponse, ApiError> {
    let transaction = Transaction::new(
        &params.transaction.from,
        &params.transaction.to,
        params.transaction.amount,
        &params.transaction.data,
        params.transaction.nonce,
        &params.transaction.signature,
    );

    if !transaction.verify(&PublicKey::from_str(&params.public_key)?) {
        return Ok(HttpResponse::BadRequest().json("Invalid public key"));
    }

    match api_tx_sender.0.send(transaction) {
        Err(error) => {
            error!("Error while submitting transaction: {:?}", error);

            Ok(HttpResponse::InternalServerError().finish())
        }
        Ok(_) => {
            info!("Transaction submitted successfully");

            Ok(HttpResponse::Ok().json("Transaction submitted successfully"))
        }
    }
}
