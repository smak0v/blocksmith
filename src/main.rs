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
use std::time::Duration;

use crate::app_state::AppState;
use crate::domain::block::Block;
use crate::event_handlers::{
    gossipsub as gossipsub_handlers, init as init_handlers, input as input_handlers,
    mdns as mdns_handlers, response as response_handlers,
};
use crate::p2p::{ChainBehaviour, ChainResponse, EventType, KEYS, LocalChainRequest, PEER_ID};
use crate::utils::telemetry;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber =
        telemetry::get_subscriber("blocksmith".to_string(), "info".to_string(), io::stdout);

    telemetry::init_subscriber(subscriber);

    info!("Node ID: {}", PEER_ID.clone());

    let (initialization_sender, mut initialization_receiver) = mpsc::unbounded_channel();
    let (chain_response_sender, mut chain_response_receiver) = mpsc::unbounded_channel();
    let (input_sender, mut input_receiver) = mpsc::unbounded_channel();
    let mut app_state = AppState::new(initialization_sender.clone(), chain_response_sender);
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
        time::sleep(Duration::from_secs(5)).await;

        info!("Sending initialization event");

        initialization_sender
            .send(true)
            .expect("cannot send initialization event");
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
                response = chain_response_receiver.recv() => {
                    Some(EventType::LocalChainResponse(response.expect("response exists")))
                },
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

                            app_state.known_peers().insert(peer_id);

                            None
                        }
                        SwarmEvent::ConnectionClosed { peer_id, ..} => {
                            info!("Connection closed with peer: {peer_id}");

                            app_state.known_peers().remove(&peer_id);

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
                    } else if let Ok(local_chain_request) =
                        serde_json::from_slice::<LocalChainRequest>(&message.data)
                    {
                        gossipsub_handlers::process_local_chain_request_message(
                            &mut app_state,
                            local_chain_request,
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
