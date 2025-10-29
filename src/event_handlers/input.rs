use tracing::error;

use std::sync::{Arc, Mutex};

use crate::app::AppState;
use crate::p2p_handlers::{
    chain as chain_handlers, peers as peer_handlers, transaction as transaction_handlers,
};

pub fn process_input(app_state: Arc<Mutex<AppState>>, input: &str) {
    match input.trim().to_lowercase().as_ref() {
        "ls c" => chain_handlers::print_chain(app_state),
        "ls t" => transaction_handlers::print_transactions(app_state),
        "ls p" => peer_handlers::print_peers(app_state),
        _ => error!("Unknown command: {}", input),
    }
}
