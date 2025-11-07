use anyhow::{Context, Result};
use libp2p::Swarm;

use crate::p2p::{
    CHAIN_TOPIC, ChainBehaviour, ChainResponse, PEER_ID, TRANSACTION_TOPIC, TransactionsResponse,
};

pub fn send_local_chain(
    swarm: &mut Swarm<ChainBehaviour>,
    chain_response: ChainResponse,
) -> Result<()> {
    if chain_response.receiver != PEER_ID.to_string() {
        let chain_response_json = serde_json::to_string(&chain_response)
            .context("failed to serialize local chain to JSON for broadcasting")?;

        swarm
            .behaviour_mut()
            .gossipsub_behaviour
            .publish(CHAIN_TOPIC.clone(), chain_response_json)
            .context("failed to publish local chain over gossipsub")?;
    }

    Ok(())
}

pub fn send_local_transactions(
    swarm: &mut Swarm<ChainBehaviour>,
    transactions_response: TransactionsResponse,
) -> Result<()> {
    if transactions_response.receiver != PEER_ID.to_string() {
        let transactions_response_json = serde_json::to_string(&transactions_response)
            .context("failed to serialize local transactions to JSON for broadcasting")?;

        swarm
            .behaviour_mut()
            .gossipsub_behaviour
            .publish(TRANSACTION_TOPIC.clone(), transactions_response_json)
            .context("failed to publish local transactions over gossipsub")?;
    }

    Ok(())
}
