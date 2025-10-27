use libp2p::Swarm;
use tracing::info;

use std::sync::{Arc, Mutex};

use crate::app_state::AppState;
use crate::domain::block::Block;
use crate::p2p::{BLOCK_TOPIC, ChainBehaviour, REMOVE_TRANSACTION_TOPIC};

pub fn add_and_broadcast_block(
    swarm: &mut Swarm<ChainBehaviour>,
    app_state: &mut Arc<Mutex<AppState>>,
    block: Block,
) {
    let mut app_state_lock = app_state.lock().expect("poisoned mutex");
    let remove_transactions_json =
        serde_json::to_string(block.transactions()).expect("cannot jsonify transactions");
    let block_json = serde_json::to_string(&block).expect("cannot jsonify block");
    let block_id = *block.id();

    app_state_lock.chain().blocks().push(block);

    info!("Broadcasting new block with id: {}", block_id);

    if app_state_lock.known_peers().len() > 0 {
        swarm
            .behaviour_mut()
            .gossipsub_behaviour
            .publish(BLOCK_TOPIC.clone(), block_json)
            .unwrap();
        swarm
            .behaviour_mut()
            .gossipsub_behaviour
            .publish(REMOVE_TRANSACTION_TOPIC.clone(), remove_transactions_json)
            .unwrap();
    }
}
