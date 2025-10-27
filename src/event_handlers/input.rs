use libp2p::Swarm;
use tracing::error;

use crate::app_state::AppState;
use crate::p2p::ChainBehaviour;
use crate::p2p_handlers::{
    block as block_handlers, chain as chain_handlers, peers as peer_handlers,
    transaction as transaction_handlers,
};

pub fn process_input(swarm: &mut Swarm<ChainBehaviour>, app_state: &mut AppState, line: &str) {
    match line.trim().to_lowercase().as_ref() {
        "ls c" => chain_handlers::print_chain(app_state),
        "ls p" => peer_handlers::print_peers(swarm),
        "ls t" => transaction_handlers::print_transactions(app_state),
        "create b" => block_handlers::create_block(swarm, app_state),
        "create t" => transaction_handlers::create_transaction(swarm, app_state),
        _ => error!("Unknown command: {}", line),
    }
}
