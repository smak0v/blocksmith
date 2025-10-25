use derive_getters::Getters;
use tokio::sync::mpsc::UnboundedSender;

use crate::chain::Chain;
use crate::p2p::ChainResponse;

#[derive(Debug, Getters)]
pub struct AppState {
    response_sender: UnboundedSender<ChainResponse>,
    init_sender: UnboundedSender<bool>,
    #[getter(skip)]
    chain: Chain,
}

impl AppState {
    pub fn new(
        response_sender: UnboundedSender<ChainResponse>,
        init_sender: UnboundedSender<bool>,
        chain: Chain,
    ) -> Self {
        Self {
            response_sender,
            init_sender,
            chain,
        }
    }

    pub fn chain(&mut self) -> &mut Chain {
        &mut self.chain
    }
}
