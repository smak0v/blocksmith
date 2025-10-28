use tracing::info;

use std::sync::{Arc, Mutex};

use crate::app_state::AppState;

pub fn print_chain(app_state: Arc<Mutex<AppState>>) {
    let mut app_state_lock = app_state.lock().expect("poisoned mutex");
    let local_chain = serde_json::to_string_pretty(app_state_lock.chain().blocks())
        .expect("cannot jsonify local chain");

    drop(app_state_lock);

    info!("Local chain:");
    info!("{}", local_chain);
}
