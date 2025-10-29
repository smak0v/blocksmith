use tokio::{sync::mpsc::UnboundedSender, time};
use tracing::{error, info};

use std::io;
use std::mem;
use std::sync::{Arc, Mutex, atomic::Ordering};
use std::thread::sleep;
use std::time::Duration;

use crate::api::launch_and_run_api_module;
use crate::app::AppState;
use crate::domain::block::Block;
use crate::domain::transaction::Transaction;

pub async fn run_node_initialization(initialization_sender: UnboundedSender<bool>) {
    time::sleep(Duration::from_secs(10)).await;

    info!("Sending initialization event");

    initialization_sender
        .send(true)
        .expect("failed to send initialization event");
}

pub fn run_input_handler(input_sender: UnboundedSender<String>) {
    let mut input = String::new();

    while io::stdin().read_line(&mut input).is_ok() {
        if !input.trim().is_empty() {
            if let Err(error) = input_sender.send(mem::take(&mut input)) {
                error!("Error sending input: {:?}", error);
            }
        } else {
            info!("Empty input");
        }
    }
}

pub async fn run_api_module(api_tx_sender: UnboundedSender<Transaction>) {
    if let Err(error) = launch_and_run_api_module(api_tx_sender).await {
        eprintln!("API crashed: {:?}", error);
    }
}

pub fn run_miner(app_state: Arc<Mutex<AppState>>, mined_block_sender: UnboundedSender<Block>) {
    let mut last_block = None;

    sleep(Duration::from_secs(10));

    loop {
        let mut app_state_lock = app_state.lock().expect("poisoned mutex");
        let prev_block;

        app_state_lock.cancel_mining.store(false, Ordering::Relaxed);

        match last_block.as_ref() {
            Some(block) => prev_block = block,
            None => {
                if app_state_lock.chain().blocks().len() == 0 {
                    continue;
                }

                prev_block = app_state_lock
                    .chain()
                    .blocks()
                    .last()
                    .expect("no previous block found")
            }
        }

        let block_id = prev_block.id() + 1;
        let prev_block_hash = prev_block.hash().clone();
        let transactions = mem::take(app_state_lock.transactions());
        let cancel_mining = app_state_lock.cancel_mining.clone();

        drop(app_state_lock);

        if let Some(new_block) = Block::new(block_id, prev_block_hash, transactions, cancel_mining)
        {
            mined_block_sender.send(new_block.clone()).unwrap();

            last_block = Some(new_block);
        }
    }
}
