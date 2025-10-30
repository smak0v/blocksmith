use tracing::{error, warn};

use std::collections::BTreeSet;
use std::sync::{Arc, atomic::AtomicBool};

use crate::domain::block::Block;
use crate::domain::transaction::Transaction;
use crate::utils::helpers;

pub const DIFFICULTY_PREFIX: &str = "000";

#[derive(Debug)]
pub struct Chain {
    blocks: Vec<Block>,
}

impl Chain {
    pub fn new(mine_genesis: bool) -> Self {
        if mine_genesis {
            let genesis_block = Block::new(
                0,
                "0000000000000000000000000000000000000000000000000000000000000000",
                BTreeSet::from([Transaction::new(
                    "0x0000000000000000000000000000000000000000",
                    "0x0000000000000000000000000000000000000000",
                    0,
                    "Genesis Block",
                    0,
                    "",
                )]),
                Arc::new(AtomicBool::new(false)),
            );

            Self {
                blocks: vec![genesis_block.unwrap()],
            }
        } else {
            Self { blocks: vec![] }
        }
    }

    pub fn blocks(&mut self) -> &mut Vec<Block> {
        &mut self.blocks
    }

    pub fn try_add_block(&mut self, block: Block) -> bool {
        let prev_block = self.blocks.last().expect("chain is empty");

        if self.is_block_valid(&block, prev_block) {
            self.blocks.push(block);

            true
        } else {
            error!("Invalid block: {:?}", block);

            false
        }
    }

    pub fn choose_chain(&mut self, local: Vec<Block>, remote: Vec<Block>) -> Vec<Block> {
        let is_local_valid = self.is_chain_valid(&local);
        let is_remote_valid = self.is_chain_valid(&remote);

        if is_local_valid && is_remote_valid {
            if remote.len() >= local.len() {
                remote
            } else {
                local
            }
        } else if is_remote_valid && !is_local_valid {
            remote
        } else if !is_remote_valid && is_local_valid {
            local
        } else {
            panic!("Local and remote chains are both invalid");
        }
    }

    fn is_block_valid(&self, block: &Block, prev_block: &Block) -> bool {
        if *block.id() != *prev_block.id() + 1 {
            warn!(
                "Block with id {} is not the next block after the latest {}",
                block.id(),
                prev_block.id()
            );

            return false;
        } else if block.prev_hash() != prev_block.hash() {
            warn!("Block with id {} has wrong previous hash", block.id());

            return false;
        } else if !helpers::hex_to_binary(&hex::decode(block.hash()).unwrap())
            .starts_with(DIFFICULTY_PREFIX)
        {
            warn!("Block with id {} has invalid difficulty", block.id());

            return false;
        } else if hex::encode(Block::calculate_hash(
            *block.id(),
            block.prev_hash(),
            *block.timestamp(),
            block.transactions(),
            *block.nonce(),
        )) != *block.hash()
        {
            warn!("Block with id {} has invalid hash", block.id());

            return false;
        }

        true
    }

    fn is_chain_valid(&self, chain: &[Block]) -> bool {
        for i in 0..chain.len() {
            if i == 0 {
                continue;
            }

            let first = chain.get(i - 1).expect("has to exist");
            let second = chain.get(i).expect("has to exist");

            if !self.is_block_valid(second, first) {
                return false;
            }
        }

        true
    }
}
