use actix_web::{
    HttpResponse,
    web::{Data, Json},
};
use secp256k1::PublicKey;
use serde::{Deserialize, Serialize};
use tracing::error;

use std::str::FromStr;

use crate::api::errors::ApiError;
use crate::api::startup::{NodeStateWrapper, SwarmWrapper};
use crate::domain::transaction::Transaction;
use crate::p2p_handlers::transaction as transaction_handlers;

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

#[tracing::instrument(name = "Submit transaction")]
pub async fn submit_transaction(
    params: Json<SubmitTransactionRequest>,
    swarm: Data<SwarmWrapper>,
    node_state: Data<NodeStateWrapper>,
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

    match swarm.0.lock() {
        Err(error) => {
            error!("Swarm mutex lock error: {:?}", error);

            Ok(HttpResponse::InternalServerError().finish())
        }
        Ok(mut swarm) => {
            transaction_handlers::add_and_broadcast_transaction(
                &mut swarm,
                node_state.0.clone(),
                transaction,
            );

            Ok(HttpResponse::Ok().finish())
        }
    }
}
