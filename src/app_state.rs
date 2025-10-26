use derive_getters::Getters;
use libp2p::PeerId;
use tokio::sync::mpsc::UnboundedSender;

use std::collections::HashSet;

use crate::chain::Chain;
use crate::p2p::ChainResponse;

#[derive(Debug, Getters)]
pub struct AppState {
    chain_response_sender: UnboundedSender<ChainResponse>,
    initialization_sender: UnboundedSender<bool>,
    #[getter(skip)]
    chain: Chain,
    #[getter(skip)]
    known_peers: HashSet<PeerId>,
}

impl AppState {
    pub fn new(
        chain_response_sender: UnboundedSender<ChainResponse>,
        initialization_sender: UnboundedSender<bool>,
        chain: Chain,
        known_peers: HashSet<PeerId>,
    ) -> Self {
        Self {
            chain_response_sender,
            initialization_sender,
            chain,
            known_peers,
        }
    }

    pub fn chain(&mut self) -> &mut Chain {
        &mut self.chain
    }

    pub fn known_peers(&mut self) -> &mut HashSet<PeerId> {
        &mut self.known_peers
    }
}
