use libp2p::Swarm;

use crate::p2p::{CHAIN_TOPIC, ChainBehaviour, Request, TRANSACTION_TOPIC};

pub fn init_node(swarm: &mut Swarm<ChainBehaviour>) {
    let chain_topic_peers_count = swarm
        .behaviour()
        .gossipsub_behaviour
        .mesh_peers(&CHAIN_TOPIC.hash())
        .count();
    let transaction_topic_peers_count = swarm
        .behaviour()
        .gossipsub_behaviour
        .mesh_peers(&TRANSACTION_TOPIC.hash())
        .count();

    if chain_topic_peers_count > 0 {
        let chain_topic_peers = swarm
            .behaviour()
            .gossipsub_behaviour
            .mesh_peers(&CHAIN_TOPIC.hash());
        let last_peer_id = chain_topic_peers.last().unwrap();
        let local_chain_request = Request {
            from_peer_id: last_peer_id.to_string(),
            topic: CHAIN_TOPIC.to_string(),
        };
        let local_chain_request_json = serde_json::to_string(&local_chain_request)
            .expect("cannot jsonify local chain request");

        swarm
            .behaviour_mut()
            .gossipsub_behaviour
            .publish(CHAIN_TOPIC.clone(), local_chain_request_json)
            .expect("cannot publish chain");
    }

    if transaction_topic_peers_count > 0 {
        let transaction_topic_peers = swarm
            .behaviour()
            .gossipsub_behaviour
            .mesh_peers(&TRANSACTION_TOPIC.hash());
        let last_peer_id = transaction_topic_peers.last().unwrap();
        let local_transactions_request = Request {
            from_peer_id: last_peer_id.to_string(),
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
