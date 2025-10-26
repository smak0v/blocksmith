use derive_getters::Getters;
use libp2p::PeerId;
use tokio::sync::mpsc::UnboundedSender;

use std::collections::HashSet;

use crate::domain::chain::Chain;
use crate::domain::transaction::Transaction;
use crate::p2p::ChainResponse;

#[derive(Debug, Getters)]
pub struct AppState {
    #[allow(dead_code)]
    initialization_sender: UnboundedSender<bool>,
    chain_response_sender: UnboundedSender<ChainResponse>,
    #[getter(skip)]
    chain: Chain,
    #[getter(skip)]
    known_peers: HashSet<PeerId>,
    #[getter(skip)]
    transactions: Vec<Transaction>,
}

impl AppState {
    pub fn new(
        initialization_sender: UnboundedSender<bool>,
        chain_response_sender: UnboundedSender<ChainResponse>,
    ) -> Self {
        Self {
            initialization_sender,
            chain_response_sender,
            chain: Chain::new(),
            known_peers: HashSet::new(),
            transactions: Vec::new(),
        }
    }

    pub fn chain(&mut self) -> &mut Chain {
        &mut self.chain
    }

    pub fn known_peers(&mut self) -> &mut HashSet<PeerId> {
        &mut self.known_peers
    }

    pub fn transactions(&mut self) -> &mut Vec<Transaction> {
        &mut self.transactions
    }
}
