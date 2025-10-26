use libp2p::Swarm;

use crate::p2p::{CHAIN_TOPIC, ChainBehaviour, ChainResponse};

pub fn send_local_chain(swarm: &mut Swarm<ChainBehaviour>, chain_response: &ChainResponse) {
    let chain_response_json =
        serde_json::to_string(chain_response).expect("cannot jsonify chain response");

    swarm
        .behaviour_mut()
        .gossipsub_behaviour
        .publish(CHAIN_TOPIC.clone(), chain_response_json)
        .expect("cannot publish chain");
}
