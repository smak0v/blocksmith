use derive_getters::Getters;
use libp2p::PeerId;
use tokio::sync::mpsc::UnboundedSender;

use std::collections::{BTreeSet, HashSet};

use crate::domain::chain::Chain;
use crate::domain::transaction::Transaction;
use crate::p2p::{ChainResponse, TransactionsResponse};

#[derive(Debug, Getters)]
pub struct AppState {
    #[allow(dead_code)]
    initialization_sender: UnboundedSender<bool>,
    chain_response_sender: UnboundedSender<ChainResponse>,
    transactions_response_sender: UnboundedSender<TransactionsResponse>,
    #[getter(skip)]
    chain: Chain,
    #[getter(skip)]
    known_peers: HashSet<PeerId>,
    #[getter(skip)]
    transactions: BTreeSet<Transaction>,
}

impl AppState {
    pub fn new(
        initialization_sender: UnboundedSender<bool>,
        chain_response_sender: UnboundedSender<ChainResponse>,
        transactions_response_sender: UnboundedSender<TransactionsResponse>,
        mine_genesis: bool,
    ) -> Self {
        Self {
            initialization_sender,
            chain_response_sender,
            transactions_response_sender,
            chain: Chain::new(mine_genesis),
            known_peers: HashSet::new(),
            transactions: BTreeSet::new(),
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
