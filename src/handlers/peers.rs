use tracing::info;

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use crate::app::AppState;

pub fn get_peers(app_state: Arc<Mutex<AppState>>) -> Vec<String> {
    let mut app_state_lock = app_state.lock().expect("poisoned mutex");
    let mut unique_peers = HashSet::new();

    for peer in app_state_lock.known_peers().iter() {
        unique_peers.insert(*peer);
    }

    drop(app_state_lock);

    unique_peers
        .into_iter()
        .map(|peer| peer.to_string())
        .collect()
}

pub fn print_peers(app_state: Arc<Mutex<AppState>>) {
    let peers = get_peers(app_state);

    if !peers.is_empty() {
        info!("Discovered peers:");

        peers.iter().for_each(|p| info!("Peer: {}", p));
    } else {
        info!("No discovered peers.");
    }
}
