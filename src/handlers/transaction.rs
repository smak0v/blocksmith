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
) {
    let transaction_json = serde_json::to_string(&transaction).expect("cannot jsonify transaction");

    info!("Broadcasting new transaction: {:?}", &transaction);

    let mut app_state_lock = app_state.lock().expect("poisoned mutex");

    app_state_lock.transactions().insert(transaction);

    if app_state_lock.known_peers().len() > 0 {
        drop(app_state_lock);

        swarm
            .behaviour_mut()
            .gossipsub_behaviour
            .publish(ADD_TRANSACTION_TOPIC.clone(), transaction_json)
            .unwrap();
    }
}

pub fn print_transactions(app_state: Arc<Mutex<AppState>>) {
    let mut app_state_lock = app_state.lock().expect("poisoned mutex");
    let local_transactions = serde_json::to_string_pretty(app_state_lock.transactions())
        .expect("cannot jsonify local transactions");

    drop(app_state_lock);

    info!("Local transactions:");
    info!("{}", local_transactions);
}
