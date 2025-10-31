use libp2p::PeerId;
use secp256k1::PublicKey;
use tokio::sync::mpsc::UnboundedSender;
use tracing::{error, info, warn};

use std::str::FromStr;
use std::sync::{Arc, Mutex, atomic::Ordering};

use crate::app::AppState;
use crate::domain::block::Block;
use crate::domain::transaction::Transaction;
use crate::p2p::{ChainResponse, PEER_ID, Request, TransactionsResponse};

pub fn process_chain_response_message(
    app_state: Arc<Mutex<AppState>>,
    chain_response: ChainResponse,
    sender: PeerId,
) {
    if chain_response.receiver == PEER_ID.to_string() {
        info!("Chain response received from: {}", sender);
        info!("Received chain length: {}", chain_response.blocks.len());

        let mut app_state_lock = app_state.lock().expect("poisoned mutex");
        let curr_chain = app_state_lock.chain().blocks().clone();

        if let Some(chain) = app_state_lock
            .chain()
            .choose_chain(curr_chain, chain_response.blocks)
        {
            *app_state_lock.chain().blocks() = chain
        }
    }
}

pub fn process_local_chain_request_message(
    app_state: Arc<Mutex<AppState>>,
    local_chain_request: Request,
    requestor: PeerId,
    chain_response_sender: UnboundedSender<ChainResponse>,
) {
    if local_chain_request.from_peer_id == PEER_ID.to_string() {
        info!("Sending local chain to: {}", requestor);

        let mut app_state_lock = app_state.lock().expect("poisoned mutex");

        if let Err(error) = chain_response_sender.send(ChainResponse {
            blocks: app_state_lock.chain().blocks().clone(),
            receiver: requestor.to_string(),
        }) {
            error!(
                "Error sending local chain response via channel, {:?}",
                error
            );
        }
    }
}

pub fn process_block_message(app_state: Arc<Mutex<AppState>>, block: Block, sender: PeerId) {
    info!(
        "Received new block with hash {} from: {}",
        block.hash(),
        sender
    );

    let transactions_to_remove = block.transactions().clone();
    let mut app_state_lock = app_state.lock().expect("poisoned mutex");
    let added = app_state_lock.chain().try_add_block(block);

    if added {
        transactions_to_remove.into_iter().for_each(|transaction| {
            app_state_lock.transactions().remove(&transaction);
        });
        app_state_lock.cancel_mining.store(true, Ordering::Relaxed);

        info!("Mining cancelled due to new block arrival");
    }
}

pub fn process_transactions_response_message(
    app_state: Arc<Mutex<AppState>>,
    transactions_response: TransactionsResponse,
    sender: PeerId,
) {
    if transactions_response.receiver == PEER_ID.to_string() {
        info!("Transactions response received from: {}", sender);
        info!(
            "Received transactions length: {}",
            transactions_response.transactions.len()
        );

        let mut app_state_lock = app_state.lock().expect("poisoned mutex");

        transactions_response
            .transactions
            .into_iter()
            .for_each(|tx| {
                if tx.verify(&PublicKey::from_str(tx.from()).expect("invalid public key")) {
                    app_state_lock.transactions().insert(tx);
                } else {
                    warn!("Received invalid transaction: {:?}", tx);
                }
            });
    }
}

pub fn process_local_transactions_request_message(
    app_state: Arc<Mutex<AppState>>,
    local_transactions_request: Request,
    requestor: PeerId,
    transactions_response_sender: UnboundedSender<TransactionsResponse>,
) {
    if local_transactions_request.from_peer_id == PEER_ID.to_string() {
        info!("Sending local transactions to: {}", requestor);

        let mut app_state_lock = app_state.lock().expect("poisoned mutex");

        if let Err(error) = transactions_response_sender.send(TransactionsResponse {
            transactions: app_state_lock.transactions().clone(),
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
    app_state: Arc<Mutex<AppState>>,
    transaction: Transaction,
    sender: PeerId,
) {
    info!("Received new transaction from: {}", sender);

    if transaction.verify(&PublicKey::from_str(transaction.from()).expect("invalid public key")) {
        let mut app_state_lock = app_state.lock().expect("poisoned mutex");

        app_state_lock.transactions().insert(transaction);
    } else {
        warn!("Received invalid transaction: {:?}", transaction);
    }
}
