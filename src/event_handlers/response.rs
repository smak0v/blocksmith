use libp2p::Swarm;

use crate::p2p::{
    CHAIN_TOPIC, ChainBehaviour, ChainResponse, PEER_ID, TRANSACTION_TOPIC, TransactionsResponse,
};

pub fn send_local_chain(swarm: &mut Swarm<ChainBehaviour>, chain_response: &ChainResponse) {
    if chain_response.receiver != PEER_ID.to_string() {
        let chain_response_json =
            serde_json::to_string(chain_response).expect("cannot jsonify chain response");

        swarm
            .behaviour_mut()
            .gossipsub_behaviour
            .publish(CHAIN_TOPIC.clone(), chain_response_json)
            .expect("cannot publish chain");
    }
}

pub fn send_local_transactions(
    swarm: &mut Swarm<ChainBehaviour>,
    transactions_response: &TransactionsResponse,
) {
    if transactions_response.receiver != PEER_ID.to_string() {
        let transactions_response_json = serde_json::to_string(transactions_response)
            .expect("cannot jsonify transactions response");

        swarm
            .behaviour_mut()
            .gossipsub_behaviour
            .publish(TRANSACTION_TOPIC.clone(), transactions_response_json)
            .expect("cannot publish transactions");
    }
}
