use libp2p::{Multiaddr, PeerId, Swarm};
use tracing::{info, warn};

use crate::p2p::ChainBehaviour;

pub fn process_mdns_discovered_event(
    swarm: &mut Swarm<ChainBehaviour>,
    discovered_peers: Vec<(PeerId, Multiaddr)>,
) {
    info!("MDNS discovered peers: {:?}", discovered_peers);

    for (peer, ..) in discovered_peers {
        info!("Dialing peer {} ...", peer);

        swarm
            .behaviour_mut()
            .gossipsub_behaviour
            .add_explicit_peer(&peer);

        match swarm.dial(peer) {
            Ok(_) => info!("Dial successful"),
            Err(error) => warn!("Failed to dial peer {}: {}", peer, error),
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
