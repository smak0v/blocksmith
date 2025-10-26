use libp2p::Swarm;
use tracing::info;

use std::mem;

use crate::p2p::{CHAIN_TOPIC, ChainBehaviour, LocalChainRequest};
use crate::p2p_handlers::peers as peers_handlers;

pub fn init_node(swarm: &mut Swarm<ChainBehaviour>) {
    let mut peers = peers_handlers::get_peers(&swarm);

    info!("Connected nodes: {}", peers.len());

    if !peers.is_empty() {
        let last_peer_id = peers.len() - 1;
        let request = LocalChainRequest {
            from_peer_id: mem::take(&mut peers[last_peer_id]),
        };
        let local_chain_request_json =
            serde_json::to_string(&request).expect("cannot jsonify request");

        swarm
            .behaviour_mut()
            .gossipsub_behaviour
            .publish(CHAIN_TOPIC.clone(), local_chain_request_json)
            .expect("cannot publish chain");
    }
}
