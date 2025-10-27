use libp2p::Swarm;
use tracing::info;

use crate::app_state::AppState;
use crate::domain::transaction::Transaction;
use crate::p2p::{ADD_TRANSACTION_TOPIC, ChainBehaviour};

pub fn create_transaction(swarm: &mut Swarm<ChainBehaviour>, app_state: &mut AppState) {
    let transaction = Transaction::new(
        "0x0000000000000000000000000000000000000000",
        "0x779D22ffB4C936ca0aeBD3ed51EF909081993991",
        1 * 10 ^ 18,
        "Transaction example",
        1,
        "",
    );
    let transaction_json = serde_json::to_string(&transaction).expect("cannot jsonify transaction");

    info!("Broadcasting new transaction: {:?}", &transaction);

    app_state.transactions().insert(transaction);

    if app_state.known_peers().len() > 0 {
        swarm
            .behaviour_mut()
            .gossipsub_behaviour
            .publish(ADD_TRANSACTION_TOPIC.clone(), transaction_json)
            .unwrap();
    }
}

pub fn print_transactions(app_state: &mut AppState) {
    let local_transactions = serde_json::to_string_pretty(app_state.transactions())
        .expect("cannot jsonify local transactions");

    info!("Local transactions:");
    info!("{}", local_transactions);
}
