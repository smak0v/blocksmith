use libp2p::{
    PeerId,
    floodsub::{Behaviour as FloodsubBehaviour, Event as FloodsubEvent, Topic},
    identity::Keypair,
    mdns::{Event as MdnsEvent, tokio::Behaviour as MdnsBehaviour},
    swarm::NetworkBehaviour,
};
use serde::{Deserialize, Serialize};

use std::sync::LazyLock;

use crate::block::Block;

pub static KEYS: LazyLock<Keypair> = LazyLock::new(|| Keypair::generate_ed25519());
pub static PEER_ID: LazyLock<PeerId> = LazyLock::new(|| PeerId::from(KEYS.public()));
pub static CHAIN_TOPIC: LazyLock<Topic> = LazyLock::new(|| Topic::new("chain"));
pub static BLOCK_TOPIC: LazyLock<Topic> = LazyLock::new(|| Topic::new("block"));

#[derive(Debug, Serialize, Deserialize)]
pub struct ChainResponse {
    pub blocks: Vec<Block>,
    pub receiver: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocalChainRequest {
    pub from_peer_id: String,
}

#[derive(Debug)]
pub enum EventType {
    Floodsub(FloodsubEvent),
    Mdns(MdnsEvent),
    LocalChainResponse(ChainResponse),
    Input(String),
    Init,
}

impl From<FloodsubEvent> for EventType {
    fn from(event: FloodsubEvent) -> Self {
        Self::Floodsub(event)
    }
}

impl From<MdnsEvent> for EventType {
    fn from(event: MdnsEvent) -> Self {
        Self::Mdns(event)
    }
}

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "EventType")]
pub struct ChainBehaviour {
    pub floodsub_behaviour: FloodsubBehaviour,
    pub mdns_behaviour: MdnsBehaviour,
}

impl ChainBehaviour {
    pub fn new() -> Self {
        let mut behaviour = Self {
            floodsub_behaviour: FloodsubBehaviour::new(*PEER_ID),
            mdns_behaviour: MdnsBehaviour::new(Default::default(), *PEER_ID)
                .expect("can not create MDNS behaviour"),
        };

        behaviour.floodsub_behaviour.subscribe(CHAIN_TOPIC.clone());
        behaviour.floodsub_behaviour.subscribe(BLOCK_TOPIC.clone());

        behaviour
    }
}
