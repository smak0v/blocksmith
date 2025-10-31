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

use std::collections::BTreeSet;
use std::sync::LazyLock;

use crate::domain::block::Block;
use crate::domain::transaction::Transaction;

pub static KEYS: LazyLock<Keypair> = LazyLock::new(Keypair::generate_ed25519);
pub static PEER_ID: LazyLock<PeerId> = LazyLock::new(|| PeerId::from(KEYS.public()));
pub static TRANSACTION_TOPIC: LazyLock<Sha256Topic> = LazyLock::new(|| Sha256Topic::new("tx"));
pub static ADD_TRANSACTION_TOPIC: LazyLock<Sha256Topic> = LazyLock::new(|| Sha256Topic::new("atx"));
pub static REMOVE_TRANSACTION_TOPIC: LazyLock<Sha256Topic> =
    LazyLock::new(|| Sha256Topic::new("rtx"));
pub static BLOCK_TOPIC: LazyLock<Sha256Topic> = LazyLock::new(|| Sha256Topic::new("block"));
pub static CHAIN_TOPIC: LazyLock<Sha256Topic> = LazyLock::new(|| Sha256Topic::new("chain"));

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "EventType")]
pub struct ChainBehaviour {
    pub gossipsub_behaviour: GossipsubBehaviour,
    pub mdns_behaviour: MdnsBehaviour,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub from_peer_id: String,
    pub topic: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChainResponse {
    pub blocks: Vec<Block>,
    pub receiver: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionsResponse {
    pub transactions: BTreeSet<Transaction>,
    pub receiver: String,
}

#[derive(Debug)]
pub enum EventType {
    Init,
    Input(String),
    LocalChainResponse(ChainResponse),
    LocalTransactionsResponse(TransactionsResponse),
    MinedBlock(Block),
    TransactionSubmitted(Transaction),
    Gossipsub(GossipsubEvent),
    Mdns(MdnsEvent),
}

impl ChainBehaviour {
    pub fn new() -> Self {
        let mut chain_behaviour = Self {
            gossipsub_behaviour: GossipsubBehaviour::new(
                MessageAuthenticity::Signed(KEYS.clone()),
                GossipsubConfig::default(),
            )
            .expect("failed to create gossipsub behaviour"),
            mdns_behaviour: MdnsBehaviour::new(Default::default(), *PEER_ID)
                .expect("failed to create MDNS behaviour"),
        };

        chain_behaviour
            .gossipsub_behaviour
            .subscribe(&TRANSACTION_TOPIC)
            .expect("failed to subscribe to TRANSACTION_TOPIC");
        chain_behaviour
            .gossipsub_behaviour
            .subscribe(&ADD_TRANSACTION_TOPIC)
            .expect("failed to subscribe to ADD_TRANSACTION_TOPIC");
        chain_behaviour
            .gossipsub_behaviour
            .subscribe(&REMOVE_TRANSACTION_TOPIC)
            .expect("failed to subscribe to REMOVE_TRANSACTION_TOPIC");
        chain_behaviour
            .gossipsub_behaviour
            .subscribe(&BLOCK_TOPIC)
            .expect("failed to subscribe to BLOCK_TOPIC");
        chain_behaviour
            .gossipsub_behaviour
            .subscribe(&CHAIN_TOPIC)
            .expect("failed to subscribe to CHAIN_TOPIC");

        chain_behaviour
    }
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
