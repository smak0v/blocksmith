use libp2p::{Multiaddr, PeerId, Swarm};
use tracing::info;

use crate::app_state::AppState;
use crate::p2p::{CHAIN_TOPIC, ChainBehaviour, LocalChainRequest};

pub fn process_mdns_discovered_event(
    swarm: &mut Swarm<ChainBehaviour>,
    app_state: &mut AppState,
    discovered_peers: Vec<(PeerId, Multiaddr)>,
) {
    info!("MDNS discovered peers: {:?}", discovered_peers);

    for (peer, ..) in discovered_peers {
        swarm
            .behaviour_mut()
            .gossipsub_behaviour
            .add_explicit_peer(&peer);
    }

    if app_state.known_peers().len() > 0 {
        for peer in app_state.known_peers().iter() {
            let local_chain_request = LocalChainRequest {
                from_peer_id: peer.to_string(),
            };
            let local_chain_request_json = serde_json::to_string(&local_chain_request)
                .expect("cannot jsonify local chain request");

            swarm
                .behaviour_mut()
                .gossipsub_behaviour
                .publish(CHAIN_TOPIC.clone(), local_chain_request_json)
                .expect("cannot publish local chain request");
        }
    }
}

pub fn process_mdns_expired_peers(
    swarm: &mut Swarm<ChainBehaviour>,
    expired_peers: Vec<(PeerId, Multiaddr)>,
) {
    info!("MDNS expired peers: {:?}", expired_peers);

    for (peer, ..) in &expired_peers {
        let discovered_peer = swarm
            .behaviour()
            .mdns_behaviour
            .discovered_nodes()
            .find(|&p| p == peer);

        if discovered_peer.is_none() {
            swarm
                .behaviour_mut()
                .gossipsub_behaviour
                .remove_explicit_peer(peer);
        }
    }
}
