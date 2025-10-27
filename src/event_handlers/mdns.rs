use libp2p::{Multiaddr, PeerId, Swarm};
use tracing::info;

use std::sync::{Arc, Mutex};

use crate::app_state::AppState;
use crate::p2p::{CHAIN_TOPIC, ChainBehaviour, Request};

pub fn process_mdns_discovered_event(
    swarm: &mut Swarm<ChainBehaviour>,
    app_state: &mut Arc<Mutex<AppState>>,
    discovered_peers: Vec<(PeerId, Multiaddr)>,
) {
    info!("MDNS discovered peers: {:?}", discovered_peers);

    for (peer, ..) in discovered_peers {
        swarm
            .behaviour_mut()
            .gossipsub_behaviour
            .add_explicit_peer(&peer);
    }

    let mut app_state_lock = app_state.lock().expect("poisoned mutex");

    if app_state_lock.known_peers().len() > 0 {
        for peer in app_state_lock.known_peers().iter() {
            let local_chain_request = Request {
                from_peer_id: peer.to_string(),
                topic: CHAIN_TOPIC.to_string(),
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
