use libp2p::Swarm;
use tracing::info;

use crate::app_state::AppState;
use crate::block::Block;
use crate::p2p::{BLOCK_TOPIC, ChainBehaviour};

pub fn create_block(cmd: &str, swarm: &mut Swarm<ChainBehaviour>, app_state: &mut AppState) {
    if let Some(data) = cmd.strip_prefix("create b") {
        let prev_block = app_state
            .chain()
            .blocks()
            .last()
            .expect("no previous block found");
        let block_id = prev_block.id() + 1;
        let block = Block::new(block_id, prev_block.hash().clone(), data.to_owned());
        let json = serde_json::to_string(&block).expect("can not jsonify block");

        app_state.chain().blocks().push(block);

        info!("Broadcasting new block with id: {}", block_id);

        swarm
            .behaviour_mut()
            .floodsub_behaviour
            .publish(BLOCK_TOPIC.clone(), json);
    }
}
