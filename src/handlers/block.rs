use anyhow::{Context, Result};
use libp2p::Swarm;
use tracing::info;

use std::sync::{Arc, Mutex};

use crate::app::AppState;
use crate::domain::block::Block;
use crate::p2p::{BLOCK_TOPIC, ChainBehaviour, REMOVE_TRANSACTION_TOPIC};

pub fn add_and_broadcast_block(
    swarm: &mut Swarm<ChainBehaviour>,
    app_state: Arc<Mutex<AppState>>,
    block: Block,
) -> Result<()> {
    let block_json = serde_json::to_string(&block)
        .context("failed to serialize block to JSON for broadcasting")?;
    let remove_transactions_json = serde_json::to_string(block.transactions())
        .context("failed to serialize block.transactions() to JSON for broadcasting")?;

    info!("Broadcasting new block with id: {}", *block.id());

    let mut app_state_lock = app_state.lock().expect("poisoned mutex");
    let should_publish = !app_state_lock.known_peers().is_empty();

    app_state_lock.chain().blocks().push(block);

    drop(app_state_lock);

    if should_publish {
        swarm
            .behaviour_mut()
            .gossipsub_behaviour
            .publish(BLOCK_TOPIC.clone(), block_json)
            .context("failed to publish block over gossipsub")?;
        swarm
            .behaviour_mut()
            .gossipsub_behaviour
            .publish(REMOVE_TRANSACTION_TOPIC.clone(), remove_transactions_json)
            .context("failed to publish transactions to remove over gossipsub")?;
    }

    Ok(())
}
