mod api;
mod app;
mod domain;
mod event_handlers;
mod p2p;
mod p2p_handlers;
mod runners;
mod swarm;
mod utils;

use anyhow::Result;
use libp2p::{
    Swarm, futures::StreamExt, gossipsub::Event as GossipsubEvent, mdns::Event as MdnsEvent,
    swarm::SwarmEvent,
};
use tokio::{select, sync::mpsc::UnboundedSender, task};
use tracing::info;

use std::sync::{Arc, Mutex};

use crate::app::{AppState, Channels};
use crate::domain::block::Block;
use crate::domain::transaction::Transaction;
use crate::event_handlers::{
    block as block_handlers, gossipsub as gossipsub_handlers, init as init_handlers,
    input as input_handlers, mdns as mdns_handlers, response as response_handlers,
};
use crate::p2p::{
    CHAIN_TOPIC, ChainBehaviour, ChainResponse, EventType, Request, TransactionsResponse,
};
use crate::p2p_handlers::transaction as transaction_handlers;
use crate::utils::telemetry;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    telemetry::init("blocksmith", "info");

    let swarm = swarm::init();
    let (app_state, channels) = app::init();
    let app_state = Arc::new(Mutex::new(app_state));

    task::spawn({
        let initialization_sender = channels.initialization.sender.clone();

        async move { runners::run_node_initialization(initialization_sender).await }
    });
    task::spawn({
        let input_sender = channels.input.sender.clone();

        async move { runners::run_input_handler(input_sender) }
    });
    task::spawn({
        let api_tx_sender = channels.api_tx.sender.clone();

        async move { runners::run_api_module(api_tx_sender).await }
    });
    task::spawn_blocking({
        let app_state = app_state.clone();
        let mined_block_sender = channels.mined_block.sender.clone();

        move || runners::run_miner(app_state, mined_block_sender)
    });

    Ok(run_node(swarm, app_state, channels).await)
}

async fn run_node(
    mut swarm: Swarm<ChainBehaviour>,
    app_state: Arc<Mutex<AppState>>,
    mut channels: Channels,
) {
    loop {
        let event = {
            select! {
                _init = channels.initialization.receiver.recv() => {
                    Some(EventType::Init)
                },
                input = channels.input.receiver.recv() => {
                    Some(EventType::Input(input.expect("cannot get input")))
                },
                chain_response = channels.chain_response.receiver.recv() => {
                    Some(EventType::LocalChainResponse(chain_response.expect("cannot get local chain response")))
                },
                txs_response = channels.txs_response.receiver.recv() => {
                    Some(EventType::LocalTransactionsResponse(txs_response.expect("cannot get local transactions response")))
                },
                mined_block_response = channels.mined_block.receiver.recv() => {
                    Some(EventType::MinedBlock(mined_block_response.expect("cannot get mined block response")))
                }
                transaction = channels.api_tx.receiver.recv() => {
                    Some(EventType::TransactionSubmitted(transaction.expect("cannot get transaction")))
                }
                swarm_event = swarm.select_next_some() => {
                    match swarm_event {
                        SwarmEvent::Behaviour(behaviour_event) => match behaviour_event {
                            EventType::Gossipsub(GossipsubEvent::Message { .. }) => Some(behaviour_event),
                            EventType::Mdns(MdnsEvent::Discovered(..)) => Some(behaviour_event),
                            EventType::Mdns(MdnsEvent::Expired(..)) => Some(behaviour_event),
                            _ => None,
                        }
                        SwarmEvent::ConnectionEstablished { peer_id, ..} => {
                            info!("Connection established with peer: {peer_id}");
                            app_state.lock().expect("poisoned mutex").known_peers().insert(peer_id);

                            None
                        }
                        SwarmEvent::ConnectionClosed { peer_id, ..} => {
                            info!("Connection closed with peer: {peer_id}");
                            app_state.lock().expect("poisoned mutex").known_peers().remove(&peer_id);

                            None
                        }
                        _ => None
                    }
                },
            }
        };

        let senders = (
            channels.chain_response.sender.clone(),
            channels.txs_response.sender.clone(),
        );

        process_event(&mut swarm, app_state.clone(), senders, event).await;
    }
}

async fn process_event(
    swarm: &mut Swarm<ChainBehaviour>,
    app_state: Arc<Mutex<AppState>>,
    senders: (
        UnboundedSender<ChainResponse>,
        UnboundedSender<TransactionsResponse>,
    ),
    event: Option<EventType>,
) {
    if let Some(event) = event {
        match event {
            EventType::Init => init_handlers::init_node(swarm, app_state).await,
            EventType::Input(input) => {
                input_handlers::process_input(app_state, input.as_ref());
            }
            EventType::LocalChainResponse(chain_response) => {
                response_handlers::send_local_chain(swarm, chain_response);
            }
            EventType::LocalTransactionsResponse(transactions_response) => {
                response_handlers::send_local_transactions(swarm, transactions_response);
            }
            EventType::MinedBlock(mined_block) => {
                block_handlers::add_and_broadcast_block(swarm, app_state, mined_block);
            }
            EventType::TransactionSubmitted(transaction) => {
                transaction_handlers::add_and_broadcast_transaction(swarm, app_state, transaction);
            }
            EventType::Mdns(MdnsEvent::Discovered(discovered_peers)) => {
                mdns_handlers::process_mdns_discovered_event(swarm, discovered_peers);
            }
            EventType::Mdns(MdnsEvent::Expired(expired_peers)) => {
                mdns_handlers::process_mdns_expired_peers(swarm, expired_peers);
            }
            EventType::Gossipsub(GossipsubEvent::Message {
                propagation_source,
                message,
                ..
            }) => {
                if let Ok(chain_response) = serde_json::from_slice::<ChainResponse>(&message.data) {
                    gossipsub_handlers::process_chain_response_message(
                        app_state,
                        chain_response,
                        propagation_source,
                    );
                } else if let Ok(request) = serde_json::from_slice::<Request>(&message.data) {
                    if request.topic == CHAIN_TOPIC.to_string() {
                        gossipsub_handlers::process_local_chain_request_message(
                            app_state,
                            request,
                            propagation_source,
                            senders.0,
                        );
                    } else {
                        gossipsub_handlers::process_local_transactions_request_message(
                            app_state,
                            request,
                            propagation_source,
                            senders.1,
                        );
                    }
                } else if let Ok(transactions_response) =
                    serde_json::from_slice::<TransactionsResponse>(&message.data)
                {
                    gossipsub_handlers::process_transactions_response_message(
                        app_state,
                        transactions_response,
                        propagation_source,
                    );
                } else if let Ok(transaction) = serde_json::from_slice::<Transaction>(&message.data)
                {
                    gossipsub_handlers::process_transaction_message(
                        app_state,
                        transaction,
                        propagation_source,
                    );
                } else if let Ok(block) = serde_json::from_slice::<Block>(&message.data) {
                    gossipsub_handlers::process_block_message(app_state, block, propagation_source);
                }
            }
            _ => {}
        }
    }
}
