use libp2p::{
    PeerId,
    gossipsub::{
        Behaviour as GossipsubBehaviour, Config as GossipsubConfig, Event as GossipsubEvent,
        MessageAuthenticity, Sha256Topic,
    },
    identity::Keypair,
    mdns::{Event as MdnsEvent, tokio::Behaviour as MdnsBehaviour},
    swarm::NetworkBehaviour,
};
use serde::{Deserialize, Serialize};

use std::sync::LazyLock;

use crate::block::Block;

pub static KEYS: LazyLock<Keypair> = LazyLock::new(|| Keypair::generate_ed25519());
pub static PEER_ID: LazyLock<PeerId> = LazyLock::new(|| PeerId::from(KEYS.public()));
pub static CHAIN_TOPIC: LazyLock<Sha256Topic> = LazyLock::new(|| Sha256Topic::new("chain"));
pub static BLOCK_TOPIC: LazyLock<Sha256Topic> = LazyLock::new(|| Sha256Topic::new("block"));

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
    Gossipsub(GossipsubEvent),
    Mdns(MdnsEvent),
    LocalChainResponse(ChainResponse),
    Input(String),
    Init,
}

impl From<GossipsubEvent> for EventType {
    fn from(event: GossipsubEvent) -> Self {
        Self::Gossipsub(event)
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
    pub gossipsub_behaviour: GossipsubBehaviour,
    pub mdns_behaviour: MdnsBehaviour,
}

impl ChainBehaviour {
    pub fn new() -> Self {
        let mut behaviour = Self {
            gossipsub_behaviour: GossipsubBehaviour::new(
                MessageAuthenticity::Signed(KEYS.clone()),
                GossipsubConfig::default(),
            )
            .unwrap(),
            mdns_behaviour: MdnsBehaviour::new(Default::default(), *PEER_ID)
                .expect("can not create MDNS behaviour"),
        };

        behaviour
            .gossipsub_behaviour
            .subscribe(&CHAIN_TOPIC)
            .unwrap();
        behaviour
            .gossipsub_behaviour
            .subscribe(&BLOCK_TOPIC)
            .unwrap();

        behaviour
    }
}
