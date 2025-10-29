use libp2p::PeerId;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use std::collections::{BTreeSet, HashSet};
use std::env;
use std::sync::{Arc, atomic::AtomicBool};

use crate::domain::block::Block;
use crate::domain::chain::Chain;
use crate::domain::transaction::Transaction;
use crate::p2p::{ChainResponse, TransactionsResponse};

pub fn init() -> (AppState, Channels) {
    (AppState::new(), Channels::new())
}

#[derive(Debug)]
pub struct AppState {
    chain: Chain,
    known_peers: HashSet<PeerId>,
    transactions: BTreeSet<Transaction>,
    pub cancel_mining: Arc<AtomicBool>,
    pub initialized: bool,
}

#[derive(Debug)]
pub struct Channels {
    pub initialization: Channel<bool>,
    pub chain_response: Channel<ChainResponse>,
    pub txs_response: Channel<TransactionsResponse>,
    pub input: Channel<String>,
    pub api_tx: Channel<Transaction>,
    pub mined_block: Channel<Block>,
}

#[derive(Debug)]
pub struct Channel<T> {
    pub sender: UnboundedSender<T>,
    pub receiver: UnboundedReceiver<T>,
}

impl AppState {
    pub fn new() -> Self {
        let mine_genesis = env::var("MINE_GENESIS")
            .expect("MINE_GENESIS environmental variable is not set")
            .trim()
            .to_lowercase();
        let mine_genesis = mine_genesis == "true";

        Self {
            chain: Chain::new(mine_genesis),
            known_peers: HashSet::new(),
            transactions: BTreeSet::new(),
            initialized: mine_genesis,
            cancel_mining: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn chain(&mut self) -> &mut Chain {
        &mut self.chain
    }

    pub fn known_peers(&mut self) -> &mut HashSet<PeerId> {
        &mut self.known_peers
    }

    pub fn transactions(&mut self) -> &mut BTreeSet<Transaction> {
        &mut self.transactions
    }
}

impl Channels {
    pub fn new() -> Self {
        Self {
            initialization: Channel::new(),
            chain_response: Channel::new(),
            txs_response: Channel::new(),
            input: Channel::new(),
            api_tx: Channel::new(),
            mined_block: Channel::new(),
        }
    }
}

impl<T> Channel<T> {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        Self { sender, receiver }
    }
}
