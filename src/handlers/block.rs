use libp2p::{Swarm, gossipsub::MessageId};
use tracing::{error, info};

use std::sync::{Arc, Mutex};

use crate::app::AppState;
use crate::domain::block::Block;
use crate::p2p::{BLOCK_TOPIC, ChainBehaviour, REMOVE_TRANSACTION_TOPIC};

pub fn add_and_broadcast_block(
    swarm: &mut Swarm<ChainBehaviour>,
    app_state: Arc<Mutex<AppState>>,
    block: Block,
) {
    let block_json = match serde_json::to_string(&block) {
        Ok(block_json) => block_json,
        Err(error) => {
            error!("Failed to serialize block to broadcast: {:?}", error);

            return;
        }
    };
    let remove_transactions_json = match serde_json::to_string(block.transactions()) {
        Ok(remove_transactions_json) => remove_transactions_json,
        Err(error) => {
            error!("Failed to serialize transactions to remove: {:?}", error);

            return;
        }
    };

    info!("Broadcasting new block with id: {}", *block.id());

    let mut app_state_lock = app_state.lock().expect("poisoned mutex");

    app_state_lock.chain().blocks().push(block);

    if app_state_lock.known_peers().len() > 0 {
        drop(app_state_lock);
        swarm
            .behaviour_mut()
            .gossipsub_behaviour
            .publish(BLOCK_TOPIC.clone(), block_json)
            .unwrap_or_else(|error| {
                error!("Error while publishing block: {:?}", error);

                MessageId(Vec::new())
            });
        swarm
            .behaviour_mut()
            .gossipsub_behaviour
            .publish(REMOVE_TRANSACTION_TOPIC.clone(), remove_transactions_json)
            .unwrap_or_else(|error| {
                error!("Error while publishing transactions to remove: {:?}", error);

                MessageId(Vec::new())
            });
    }
}
