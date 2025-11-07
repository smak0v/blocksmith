use anyhow::{Context, Result};
use libp2p::Swarm;
use tracing::info;

use std::sync::{Arc, Mutex};

use crate::app::AppState;
use crate::domain::transaction::Transaction;
use crate::p2p::{ADD_TRANSACTION_TOPIC, ChainBehaviour};

pub fn add_and_broadcast_transaction(
    swarm: &mut Swarm<ChainBehaviour>,
    app_state: Arc<Mutex<AppState>>,
    transaction: Transaction,
) -> Result<()> {
    let transaction_json = serde_json::to_string(&transaction)
        .context("failed to serialize transaction to JSON for broadcasting")?;

    info!("Broadcasting new transaction: {:?}", &transaction);

    let mut app_state_lock = app_state.lock().expect("poisoned mutex");
    let should_broadcast = !app_state_lock.known_peers().is_empty();

    app_state_lock.transactions().insert(transaction);

    drop(app_state_lock);

    if should_broadcast {
        swarm
            .behaviour_mut()
            .gossipsub_behaviour
            .publish(ADD_TRANSACTION_TOPIC.clone(), transaction_json)
            .context("failed to publish transaction over gossipsub")?;
    }

    Ok(())
}

pub fn print_transactions(app_state: Arc<Mutex<AppState>>) -> Result<()> {
    let local_transactions = {
        let mut app_state_lock = app_state.lock().expect("poisoned mutex");

        serde_json::to_string_pretty(app_state_lock.transactions())
            .context("failed to serialize transactions to JSON for printing")?
    };

    info!("Local transactions:");
    info!("{}", local_transactions);

    Ok(())
}
