use tracing::info;

use crate::app_state::AppState;

pub fn print_chain(app_state: &mut AppState) {
    let local_chain = serde_json::to_string_pretty(app_state.chain().blocks())
        .expect("cannot jsonify local chain");

    info!("Local chain:");
    info!("{}", local_chain);
}
