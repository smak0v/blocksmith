use libp2p::{Swarm, gossipsub::MessageId};
use tracing::{error, info};

use std::sync::{Arc, Mutex};

use crate::app::AppState;
use crate::domain::transaction::Transaction;
use crate::p2p::{ADD_TRANSACTION_TOPIC, ChainBehaviour};

pub fn add_and_broadcast_transaction(
    swarm: &mut Swarm<ChainBehaviour>,
    app_state: Arc<Mutex<AppState>>,
    transaction: Transaction,
) {
    let transaction_json = match serde_json::to_string(&transaction) {
        Ok(transaction_json) => transaction_json,
        Err(error) => {
            error!("Failed to serialize transaction to broadcast: {:?}", error);

            return;
        }
    };

    info!("Broadcasting new transaction: {:?}", &transaction);

    let mut app_state_lock = app_state.lock().expect("poisoned mutex");

    app_state_lock.transactions().insert(transaction);

    if app_state_lock.known_peers().len() > 0 {
        drop(app_state_lock);
        swarm
            .behaviour_mut()
            .gossipsub_behaviour
            .publish(ADD_TRANSACTION_TOPIC.clone(), transaction_json)
            .unwrap_or_else(|error| {
                error!("Error while publishing transaction: {:?}", error);

                MessageId(Vec::new())
            });
    }
}

pub fn print_transactions(app_state: Arc<Mutex<AppState>>) {
    let local_transactions = {
        let mut app_state_lock = app_state.lock().expect("poisoned mutex");

        match serde_json::to_string_pretty(app_state_lock.transactions()) {
            Ok(local_transactions) => local_transactions,
            Err(error) => {
                error!("Error serializing local transactions: {:?}", error);

                return;
            }
        }
    };

    info!("Local transactions:");
    info!("{}", local_transactions);
}
