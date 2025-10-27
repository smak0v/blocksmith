use libp2p::Swarm;
use tracing::info;

use std::mem;

use crate::app_state::AppState;
use crate::domain::block::Block;
use crate::p2p::{BLOCK_TOPIC, ChainBehaviour, REMOVE_TRANSACTION_TOPIC};

pub fn create_block(swarm: &mut Swarm<ChainBehaviour>, app_state: &mut AppState) {
    let remove_transactions_json =
        serde_json::to_string(app_state.transactions()).expect("cannot jsonify transactions");
    let prev_block = app_state
        .chain()
        .blocks()
        .last()
        .expect("no previous block found");
    let block_id = prev_block.id() + 1;
    let block = Block::new(
        block_id,
        prev_block.hash().clone(),
        mem::take(app_state.transactions()),
    );
    let block_json = serde_json::to_string(&block).expect("cannot jsonify block");

    app_state.chain().blocks().push(block);

    info!("Broadcasting new block with id: {}", block_id);

    if app_state.known_peers().len() > 0 {
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
