use tracing::info;

use crate::app_state::AppState;

pub fn print_chain(app_state: &mut AppState) {
    info!("Local chain:");

    let local_chain =
        serde_json::to_string_pretty(app_state.chain().blocks()).expect("can not jsonify blocks");

    info!("{}", local_chain);
}
