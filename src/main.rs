mod api;
mod app_state;
mod domain;
mod event_handlers;
mod p2p;
mod p2p_handlers;
mod runners;
mod utils;

use anyhow::Result;
use libp2p::{
    Swarm, SwarmBuilder, futures::StreamExt, gossipsub::Event as GossipsubEvent,
    mdns::Event as MdnsEvent, noise::Config as NoiseConfig, swarm::SwarmEvent,
    tcp::Config as TcpConfig, tls::Config as TlsConfig, yamux::Config as YamuxConfig,
};
use tokio::{select, sync::mpsc, task};
use tracing::info;

use std::env;
use std::sync::{Arc, Mutex};

use crate::app_state::AppState;
use crate::domain::block::Block;
use crate::domain::transaction::Transaction;
use crate::event_handlers::{
    block as block_handlers, gossipsub as gossipsub_handlers, init as init_handlers,
    input as input_handlers, mdns as mdns_handlers, response as response_handlers,
};
use crate::p2p::{
    CHAIN_TOPIC, ChainBehaviour, ChainResponse, EventType, KEYS, PEER_ID, Request,
    TransactionsResponse,
};
use crate::p2p_handlers::transaction as transaction_handlers;
use crate::utils::telemetry;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    telemetry::init("blocksmith", "info");

    info!("Node ID: {}", PEER_ID.clone());

    let (initialization_sender, mut initialization_receiver) = mpsc::unbounded_channel();
    let (chain_response_sender, mut chain_response_receiver) = mpsc::unbounded_channel();
    let (txs_response_sender, mut txs_response_receiver) = mpsc::unbounded_channel();
    let (input_sender, mut input_receiver) = mpsc::unbounded_channel();
    let (api_tx_sender, mut api_tx_receiver) = mpsc::unbounded_channel();
    let (mined_block_sender, mut mined_block_receiver) = mpsc::unbounded_channel();

    let mine_genesis_block = env::var("MINE_GENESIS")
        .expect("MINE_GENESIS environmental variable is not set")
        .trim()
        .to_lowercase();
    let app_state = Arc::new(Mutex::new(AppState::new(
        initialization_sender.clone(),
        chain_response_sender,
        txs_response_sender,
        mine_genesis_block == "true",
    )));
    let mut swarm = SwarmBuilder::with_existing_identity(KEYS.clone())
        .with_tokio()
        .with_tcp(
            TcpConfig::default(),
            (TlsConfig::new, NoiseConfig::new),
            YamuxConfig::default,
        )?
        .with_behaviour(|_| ChainBehaviour::new())?
        .with_swarm_config(|cfg| cfg)
        .build();

    Swarm::listen_on(
        &mut swarm,
        "/ip4/0.0.0.0/tcp/0"
            .parse()
            .expect("cannot get local socket"),
    )
    .expect("swarm cannot be started");

    task::spawn(async move { runners::run_node_initialization(initialization_sender).await });
    task::spawn(async move { runners::run_input_handler(input_sender) });
    task::spawn(async move { runners::run_api_module(api_tx_sender).await });
    task::spawn_blocking({
        let app_state = app_state.clone();

        move || runners::run_miner(app_state, mined_block_sender)
    });

    loop {
        let event = {
            select! {
                _init = initialization_receiver.recv() => {
                    Some(EventType::Init)
                },
                input = input_receiver.recv() => {
                    Some(EventType::Input(input.expect("cannot get input")))
                },
                chain_response = chain_response_receiver.recv() => {
                    Some(EventType::LocalChainResponse(chain_response.expect("cannot get local chain response")))
                },
                txs_response = txs_response_receiver.recv() => {
                    Some(EventType::LocalTransactionsResponse(txs_response.expect("cannot get local transactions response")))
                },
                mined_block_response = mined_block_receiver.recv() => {
                    Some(EventType::MinedBlock(mined_block_response.expect("cannot get mined block response")))
                }
                transaction = api_tx_receiver.recv() => {
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

        process_event(&mut swarm, app_state.clone(), event);
    }
}

fn process_event(
    swarm: &mut Swarm<ChainBehaviour>,
    app_state: Arc<Mutex<AppState>>,
    event: Option<EventType>,
) {
    if let Some(event) = event {
        match event {
            EventType::Init => init_handlers::init_node(swarm),
            EventType::Input(input) => {
                input_handlers::process_input(swarm, app_state, input.as_ref());
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
                mdns_handlers::process_mdns_discovered_event(swarm, app_state, discovered_peers);
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
                        );
                    } else {
                        gossipsub_handlers::process_local_transactions_request_message(
                            app_state,
                            request,
                            propagation_source,
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
