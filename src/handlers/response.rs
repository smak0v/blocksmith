use libp2p::{Swarm, gossipsub::MessageId};
use tracing::error;

use crate::p2p::{
    CHAIN_TOPIC, ChainBehaviour, ChainResponse, PEER_ID, TRANSACTION_TOPIC, TransactionsResponse,
};

pub fn send_local_chain(swarm: &mut Swarm<ChainBehaviour>, chain_response: ChainResponse) {
    if chain_response.receiver != PEER_ID.to_string() {
        let chain_response_json = match serde_json::to_string(&chain_response) {
            Ok(chain_response_json) => chain_response_json,
            Err(error) => {
                error!("Error serializing local chain to JSON: {:?}", error);

                return;
            }
        };

        swarm
            .behaviour_mut()
            .gossipsub_behaviour
            .publish(CHAIN_TOPIC.clone(), chain_response_json)
            .unwrap_or_else(|error| {
                error!("Error while publishing local chain: {:?}", error);

                MessageId(Vec::new())
            });
    }
}

pub fn send_local_transactions(
    swarm: &mut Swarm<ChainBehaviour>,
    transactions_response: TransactionsResponse,
) {
    if transactions_response.receiver != PEER_ID.to_string() {
        let transactions_response_json = match serde_json::to_string(&transactions_response) {
            Ok(transactions_response_json) => transactions_response_json,
            Err(error) => {
                error!("Error serializing local transactions to JSON: {:?}", error);

                return;
            }
        };

        swarm
            .behaviour_mut()
            .gossipsub_behaviour
            .publish(TRANSACTION_TOPIC.clone(), transactions_response_json)
            .unwrap_or_else(|error| {
                error!("Error while publishing local transactions: {:?}", error);

                MessageId(Vec::new())
            });
    }
}
