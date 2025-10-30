use tracing::{error, info};

use std::sync::{Arc, Mutex};

use crate::app::AppState;

pub fn print_chain(app_state: Arc<Mutex<AppState>>) {
    let local_chain = {
        let mut app_state_lock = app_state.lock().expect("poisoned mutex");

        match serde_json::to_string_pretty(app_state_lock.chain().blocks()) {
            Ok(local_chain) => local_chain,
            Err(error) => {
                error!("Error serializing local chain: {:?}", error);

                return;
            }
        }
    };

    info!("Local chain:");
    info!("{}", local_chain);
}
