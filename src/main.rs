mod app_state;
mod block;
mod chain;
mod handlers;
mod p2p;
mod telemetry;
mod utils;

use anyhow::Result;
use libp2p::{
    Swarm, SwarmBuilder, futures::StreamExt, gossipsub::Event as GossipsubEvent,
    mdns::Event as MdnsEvent, noise::Config as NoiseConfig, swarm::SwarmEvent,
    tcp::Config as TcpConfig, tls::Config as TlsConfig, yamux::Config as YamuxConfig,
};
use std::collections::HashSet;
use tokio::{select, sync::mpsc, time};
use tracing::{debug, error, info};

use std::io;
use std::mem;
use std::time::Duration;

use crate::app_state::AppState;
use crate::block::Block;
use crate::chain::Chain;
use crate::handlers::{block as block_handlers, chain as chain_handlers, peers as peers_handlers};
use crate::p2p::{
    CHAIN_TOPIC, ChainBehaviour, ChainResponse, EventType, KEYS, LocalChainRequest, PEER_ID,
};

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber =
        telemetry::get_subscriber("blocksmith".to_string(), "info".to_string(), io::stdout);

    telemetry::init_subscriber(subscriber);

    info!("Peer Id: {}", PEER_ID.clone());

    let (response_sender, mut response_rcv) = mpsc::unbounded_channel();
    let (init_sender, mut init_rcv) = mpsc::unbounded_channel();
    let (input_tx, mut input_rx) = mpsc::unbounded_channel();

    let mut app_state = AppState::new(
        response_sender,
        init_sender.clone(),
        Chain::new(),
        HashSet::new(),
    );
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
                if let Err(error) = input_tx.send(mem::take(&mut input)) {
                    error!("Error sending input: {:?}", error);
                }
            }
        }
    });

    Swarm::listen_on(
        &mut swarm,
        "/ip4/0.0.0.0/tcp/0"
            .parse()
            .expect("can get a local socket"),
    )
    .expect("swarm can be started");

    tokio::spawn(async move {
        time::sleep(Duration::from_secs(5)).await;

        info!("sending init event");

        init_sender.send(true).expect("can send init event");
    });

    loop {
        let evt = {
            select! {
                line = input_rx.recv() => {
                    Some(EventType::Input(line.expect("can get line")))
                }
                response = response_rcv.recv() => {
                    Some(EventType::LocalChainResponse(response.expect("response exists")))
                },
                _init = init_rcv.recv() => {
                    Some(EventType::Init)
                }
                event = swarm.select_next_some() => {
                    match event {
                        SwarmEvent::Behaviour(event) => match event {
                            EventType::Mdns(MdnsEvent::Discovered(..)) =>  Some(event),
                            EventType::Mdns(MdnsEvent::Expired(..)) => Some(event),
                            EventType::Gossipsub(GossipsubEvent::Message { .. }) => Some(event),
                            _ => None,
                        }
                        SwarmEvent::ConnectionEstablished { peer_id, ..} => {
                            app_state.known_peers().insert(peer_id);

                            None
                        }
                        SwarmEvent::ConnectionClosed { peer_id, ..} => {
                            app_state.known_peers().remove(&peer_id);

                            None
                        }
                        _ => None
                    }
                },
            }
        };

        if let Some(event) = evt {
            match event {
                EventType::Init => {
                    let peers = peers_handlers::get_peers(&swarm);

                    app_state.chain().genesis();

                    info!("connected nodes: {}", peers.len());

                    if !peers.is_empty() {
                        let req = LocalChainRequest {
                            from_peer_id: peers
                                .iter()
                                .last()
                                .expect("at least one peer")
                                .to_string(),
                        };
                        let json = serde_json::to_string(&req).expect("can not jsonify request");

                        swarm
                            .behaviour_mut()
                            .gossipsub_behaviour
                            .publish(CHAIN_TOPIC.clone(), json)?;
                    }
                }
                EventType::LocalChainResponse(resp) => {
                    let json = serde_json::to_string(&resp).expect("can not jsonify response");
                    swarm
                        .behaviour_mut()
                        .gossipsub_behaviour
                        .publish(CHAIN_TOPIC.clone(), json)?;
                }
                EventType::Input(line) => match line.trim() {
                    "ls p" => peers_handlers::print_peers(&swarm),
                    cmd if cmd.starts_with("ls c") => chain_handlers::print_chain(&mut app_state),
                    cmd if cmd.starts_with("create b") => {
                        block_handlers::create_block(cmd, &mut swarm, &mut app_state)
                    }
                    _ => {
                        error!("unknown command");
                    }
                },
                EventType::Mdns(MdnsEvent::Discovered(discovered_list)) => {
                    debug!("MDNS discovered list: {:#?}", discovered_list);

                    for (peer, ..) in &discovered_list {
                        swarm
                            .behaviour_mut()
                            .gossipsub_behaviour
                            .add_explicit_peer(peer);
                    }

                    if app_state.known_peers().len() > 0 {
                        for peer in app_state.known_peers().iter() {
                            let req = LocalChainRequest {
                                from_peer_id: peer.to_string(),
                            };
                            let json = serde_json::to_string(&req)?;

                            swarm
                                .behaviour_mut()
                                .gossipsub_behaviour
                                .publish(CHAIN_TOPIC.clone(), json)?;
                        }
                    }
                }
                EventType::Mdns(MdnsEvent::Expired(expired_list)) => {
                    debug!("MDNS expired list: {:#?}", expired_list);

                    for (peer, _addr) in &expired_list {
                        if swarm
                            .behaviour()
                            .mdns_behaviour
                            .discovered_nodes()
                            .find(|&p| p == peer)
                            .is_none()
                        {
                            swarm
                                .behaviour_mut()
                                .gossipsub_behaviour
                                .remove_explicit_peer(peer);
                        }
                    }
                }
                EventType::Gossipsub(GossipsubEvent::Message {
                    propagation_source,
                    message_id: _,
                    message,
                }) => {
                    if let Ok(resp) = serde_json::from_slice::<ChainResponse>(&message.data) {
                        if resp.receiver == PEER_ID.to_string() {
                            info!("Response from {}:", propagation_source);

                            resp.blocks.iter().for_each(|r| info!("{:?}", r));

                            let curr_blocks = app_state.chain().blocks().clone();

                            *app_state.chain().blocks() =
                                app_state.chain().choose_chain(curr_blocks, resp.blocks);
                        }
                    } else if let Ok(resp) =
                        serde_json::from_slice::<LocalChainRequest>(&message.data)
                    {
                        info!("sending local chain to {}", propagation_source);

                        let peer_id = resp.from_peer_id;

                        if PEER_ID.to_string() == peer_id {
                            let response_sender = app_state.chain_response_sender().clone();

                            if let Err(e) = response_sender.send(ChainResponse {
                                blocks: app_state.chain().blocks().clone(),
                                receiver: propagation_source.to_string(),
                            }) {
                                error!("error sending response via channel, {}", e);
                            }
                        }
                    } else if let Ok(block) = serde_json::from_slice::<Block>(&message.data) {
                        info!("received new block from {}", propagation_source);

                        app_state.chain().try_add_block(block);
                    }
                }
                _ => {}
            }
        }
    }
}
