use libp2p::Swarm;
use tracing::info;

use std::collections::HashSet;

use crate::p2p::ChainBehaviour;

pub fn get_peers(swarm: &Swarm<ChainBehaviour>) -> Vec<String> {
    let peers = swarm.behaviour().mdns_behaviour.discovered_nodes();
    let mut unique_peers = HashSet::new();

    for peer in peers {
        unique_peers.insert(peer);
    }

    unique_peers
        .into_iter()
        .map(|&peer| peer.to_string())
        .collect()
}

pub fn print_peers(swarm: &Swarm<ChainBehaviour>) {
    let peers = get_peers(swarm);

    if !peers.is_empty() {
        info!("Discovered peers:");

        peers.iter().for_each(|p| info!("Peer: {}", p));
    } else {
        info!("No discovered peers.");
    }
}
