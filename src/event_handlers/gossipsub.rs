use libp2p::PeerId;
use tracing::{error, info};

use crate::app_state::AppState;
use crate::domain::block::Block;
use crate::p2p::{ChainResponse, LocalChainRequest, PEER_ID};

pub fn process_chain_response_message(
    app_state: &mut AppState,
    chain_response: ChainResponse,
    sender: PeerId,
) {
    if chain_response.receiver == PEER_ID.to_string() {
        info!("Chain response received from: {}", sender);
        info!("Received chain:");

        chain_response
            .blocks
            .iter()
            .for_each(|block| info!("{:#?}", block));

        let curr_chain = app_state.chain().blocks().clone();

        *app_state.chain().blocks() = app_state
            .chain()
            .choose_chain(curr_chain, chain_response.blocks);
    }
}

pub fn process_local_chain_request_message(
    app_state: &mut AppState,
    local_chain_request: LocalChainRequest,
    requestor: PeerId,
) {
    if local_chain_request.from_peer_id == PEER_ID.to_string() {
        info!("Sending local chain to: {}", requestor);

        let chain_response_sender = app_state.chain_response_sender().clone();

        if let Err(error) = chain_response_sender.send(ChainResponse {
            blocks: app_state.chain().blocks().clone(),
            receiver: requestor.to_string(),
        }) {
            error!(
                "Error sending local chain response via channel, {:?}",
                error
            );
        }
    }
}

pub fn process_block_message(app_state: &mut AppState, block: Block, sender: PeerId) {
    info!("Received new block from: {}", sender);

    app_state.chain().try_add_block(block);
}
