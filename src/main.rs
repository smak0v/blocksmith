mod app_state;
mod domain;
mod event_handlers;
mod p2p;
mod p2p_handlers;
mod utils;

use anyhow::Result;
use libp2p::{
    Swarm, SwarmBuilder, futures::StreamExt, gossipsub::Event as GossipsubEvent,
    mdns::Event as MdnsEvent, noise::Config as NoiseConfig, swarm::SwarmEvent,
    tcp::Config as TcpConfig, tls::Config as TlsConfig, yamux::Config as YamuxConfig,
};
use tokio::{select, sync::mpsc, time};
use tracing::{error, info};

use std::io;
use std::mem;
use std::sync::{Arc, Mutex};
use std::time::Duration;

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
use crate::utils::telemetry;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber =
        telemetry::get_subscriber("blocksmith".to_string(), "info".to_string(), io::stdout);

    telemetry::init_subscriber(subscriber);

    info!("Node ID: {}", PEER_ID.clone());

    let (initialization_sender, mut initialization_receiver) = mpsc::unbounded_channel();
    let (chain_response_sender, mut chain_response_receiver) = mpsc::unbounded_channel();
    let (transactions_response_sender, mut transactions_response_receiver) =
        mpsc::unbounded_channel();
    let (mined_block_sender, mut mined_block_receiver) = mpsc::unbounded_channel();
    let (input_sender, mut input_receiver) = mpsc::unbounded_channel();
    let mut app_state = Arc::new(Mutex::new(AppState::new(
        initialization_sender.clone(),
        chain_response_sender,
        transactions_response_sender,
        mined_block_sender.clone(),
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

    tokio::spawn(async move {
        let mut input = String::new();

        while io::stdin().read_line(&mut input).is_ok() {
            if !input.trim().is_empty() {
                if let Err(error) = input_sender.send(mem::take(&mut input)) {
                    error!("Error sending input: {:?}", error);
                }
            }
        }
    });

    Swarm::listen_on(
        &mut swarm,
        "/ip4/0.0.0.0/tcp/0"
            .parse()
            .expect("cannot get local socket"),
    )
    .expect("swarm cannot be started");

    tokio::spawn(async move {
        time::sleep(Duration::from_secs(10)).await;

        info!("Sending initialization event");

        initialization_sender
            .send(true)
            .expect("cannot send initialization event");
    });

    tokio::task::spawn_blocking({
        let app_state = app_state.clone();

        move || {
            let mut last_block = None;

            loop {
                let mut app_state_lock = app_state.lock().expect("poisoned mutex");
                let prev_block;

                match last_block.as_ref() {
                    Some(block) => prev_block = block,
                    None => {
                        if app_state_lock.chain().blocks().len() == 0 {
                            continue;
                        }

                        prev_block = app_state_lock
                            .chain()
                            .blocks()
                            .last()
                            .expect("no previous block found")
                    }
                }

                let block_id = prev_block.id() + 1;
                let prev_block_hash = prev_block.hash().clone();
                let transactions = mem::take(app_state_lock.transactions());

                drop(app_state_lock);

                let new_block = Block::new(block_id, prev_block_hash, transactions);

                mined_block_sender.send(new_block.clone()).unwrap();

                last_block = Some(new_block);
            }
        }
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
                transactions_response = transactions_response_receiver.recv() => {
                    Some(EventType::LocalTransactionsResponse(transactions_response.expect("cannot get local transactions response")))
                },
                mined_block_response = mined_block_receiver.recv() => {
                    Some(EventType::MinedBlock(mined_block_response.expect("cannot get mined block response")))
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

        if let Some(event) = event {
            match event {
                EventType::Init => init_handlers::init_node(&mut swarm),
                EventType::Input(line) => {
                    input_handlers::process_input(&mut swarm, &mut app_state, &line);
                }
                EventType::LocalChainResponse(chain_response) => {
                    response_handlers::send_local_chain(&mut swarm, &chain_response);
                }
                EventType::LocalTransactionsResponse(transactions_response) => {
                    response_handlers::send_local_transactions(&mut swarm, &transactions_response);
                }
                EventType::MinedBlock(mined_block) => {
                    block_handlers::add_and_broadcast_block(
                        &mut swarm,
                        &mut app_state,
                        mined_block,
                    );
                }
                EventType::Mdns(MdnsEvent::Discovered(discovered_peers)) => {
                    mdns_handlers::process_mdns_discovered_event(
                        &mut swarm,
                        &mut app_state,
                        discovered_peers,
                    );
                }
                EventType::Mdns(MdnsEvent::Expired(expired_peers)) => {
                    mdns_handlers::process_mdns_expired_peers(&mut swarm, expired_peers);
                }
                EventType::Gossipsub(GossipsubEvent::Message {
                    propagation_source,
                    message,
                    ..
                }) => {
                    if let Ok(chain_response) =
                        serde_json::from_slice::<ChainResponse>(&message.data)
                    {
                        gossipsub_handlers::process_chain_response_message(
                            &mut app_state,
                            chain_response,
                            propagation_source,
                        );
                    } else if let Ok(request) = serde_json::from_slice::<Request>(&message.data) {
                        if request.topic == CHAIN_TOPIC.to_string() {
                            gossipsub_handlers::process_local_chain_request_message(
                                &mut app_state,
                                request,
                                propagation_source,
                            );
                        } else {
                            gossipsub_handlers::process_local_transactions_request_message(
                                &mut app_state,
                                request,
                                propagation_source,
                            );
                        }
                    } else if let Ok(transactions_response) =
                        serde_json::from_slice::<TransactionsResponse>(&message.data)
                    {
                        gossipsub_handlers::process_transactions_response_message(
                            &mut app_state,
                            transactions_response,
                            propagation_source,
                        );
                    } else if let Ok(transaction) =
                        serde_json::from_slice::<Transaction>(&message.data)
                    {
                        gossipsub_handlers::process_transaction_message(
                            &mut app_state,
                            transaction,
                            propagation_source,
                        );
                    } else if let Ok(block) = serde_json::from_slice::<Block>(&message.data) {
                        gossipsub_handlers::process_block_message(
                            &mut app_state,
                            block,
                            propagation_source,
                        );
                    }
                }
                _ => {}
            }
        }
    }
}
