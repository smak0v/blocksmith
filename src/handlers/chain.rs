use anyhow::{Context, Result};
use tracing::info;

use std::sync::{Arc, Mutex};

use crate::app::AppState;

pub fn print_chain(app_state: Arc<Mutex<AppState>>) -> Result<()> {
    let local_chain = {
        let mut app_state_lock = app_state.lock().expect("poisoned mutex");

        serde_json::to_string_pretty(app_state_lock.chain().blocks())
            .context("failed to serialize chain to JSON for printing")?
    };

    info!("Local chain:");
    info!("{}", local_chain);

    Ok(())
}
