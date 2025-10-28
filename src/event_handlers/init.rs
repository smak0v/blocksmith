use libp2p::Swarm;

use std::mem;
use std::thread::sleep;
use std::time::Duration;

use crate::p2p::{CHAIN_TOPIC, ChainBehaviour, Request, TRANSACTION_TOPIC};
use crate::p2p_handlers::peers as peers_handlers;

pub fn init_node(swarm: &mut Swarm<ChainBehaviour>) {
    let mut peers = peers_handlers::get_peers(&swarm);

    if !peers.is_empty() {
        let last_peer_id = peers.len() - 1;
        let last_peer = mem::take(&mut peers[last_peer_id]);

        sleep(Duration::from_secs(10));

        let local_chain_request = Request {
            from_peer_id: last_peer.to_string(),
            topic: CHAIN_TOPIC.to_string(),
        };
        let local_chain_request_json = serde_json::to_string(&local_chain_request)
            .expect("cannot jsonify local chain request");

        swarm
            .behaviour_mut()
            .gossipsub_behaviour
            .publish(CHAIN_TOPIC.clone(), local_chain_request_json)
            .expect("cannot publish chain");

        let local_transactions_request = Request {
            from_peer_id: last_peer.to_string(),
            topic: TRANSACTION_TOPIC.to_string(),
        };
        let local_transactions_request_json = serde_json::to_string(&local_transactions_request)
            .expect("cannot jsonify local transactions request");

        swarm
            .behaviour_mut()
            .gossipsub_behaviour
            .publish(TRANSACTION_TOPIC.clone(), local_transactions_request_json)
            .expect("cannot publish transactions");
    }
}
