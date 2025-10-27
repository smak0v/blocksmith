use libp2p::PeerId;
use secp256k1::PublicKey;
use tracing::{error, info, warn};

use std::str::FromStr;

use crate::app_state::AppState;
use crate::domain::block::Block;
use crate::domain::transaction::Transaction;
use crate::p2p::{ChainResponse, PEER_ID, Request, TransactionsResponse};

pub fn process_chain_response_message(
    app_state: &mut AppState,
    chain_response: ChainResponse,
    sender: PeerId,
) {
    if chain_response.receiver == PEER_ID.to_string() {
        info!("Chain response received from: {}", sender);
        info!("Received chain length: {}", chain_response.blocks.len());

        let curr_chain = app_state.chain().blocks().clone();

        *app_state.chain().blocks() = app_state
            .chain()
            .choose_chain(curr_chain, chain_response.blocks);
    }
}

pub fn process_local_chain_request_message(
    app_state: &mut AppState,
    local_chain_request: Request,
    requestor: PeerId,
) {
    if local_chain_request.from_peer_id == PEER_ID.to_string() {
        info!("Sending local chain to: {}", requestor);

        let chain_response_sender = app_state.chain_response_sender().clone();

        if let Err(error) = chain_response_sender.send(ChainResponse {
            blocks: app_state.chain().blocks().clone(),
            receiver: requestor.to_string(),
        }) {
            error!(
                "Error sending local chain response via channel, {:?}",
                error
            );
        }
    }
}

pub fn process_block_message(app_state: &mut AppState, block: Block, sender: PeerId) {
    info!("Received new block from: {}", sender);

    let transactions_to_remove = block.transactions().clone();
    let added = app_state.chain().try_add_block(block);

    if added {
        transactions_to_remove.into_iter().for_each(|transaction| {
            app_state.transactions().remove(&transaction);
        })
    }
}

pub fn process_transactions_response_message(
    app_state: &mut AppState,
    transactions_response: TransactionsResponse,
    sender: PeerId,
) {
    if transactions_response.receiver == PEER_ID.to_string() {
        info!("Transactions response received from: {}", sender);
        info!(
            "Received transactions length: {}",
            transactions_response.transactions.len()
        );

        transactions_response
            .transactions
            .into_iter()
            .for_each(|tx| {
                if tx.verify(&PublicKey::from_str(tx.from()).expect("invalid public key")) {
                    app_state.transactions().insert(tx);
                } else {
                    warn!("Received invalid transaction: {:?}", tx);
                }
            });
    }
}

pub fn process_local_transactions_request_message(
    app_state: &mut AppState,
    local_transactions_request: Request,
    requestor: PeerId,
) {
    if local_transactions_request.from_peer_id == PEER_ID.to_string() {
        info!("Sending local transactions to: {}", requestor);

        let transactions_response_sender = app_state.transactions_response_sender().clone();

        if let Err(error) = transactions_response_sender.send(TransactionsResponse {
            transactions: app_state.transactions().clone(),
            receiver: requestor.to_string(),
        }) {
            error!(
                "Error sending local transactions response via channel, {:?}",
                error
            );
        }
    }
}

pub fn process_transaction_message(
    app_state: &mut AppState,
    transaction: Transaction,
    sender: PeerId,
) {
    info!("Received new transaction from: {}", sender);

    if transaction.verify(&PublicKey::from_str(transaction.from()).expect("invalid public key")) {
        app_state.transactions().insert(transaction);
    } else {
        warn!("Received invalid transaction: {:?}", transaction);
    }
}
