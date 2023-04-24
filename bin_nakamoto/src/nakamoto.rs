// This file is part of the project for the module CS3235 by Prateek
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

// This file implements the Nakamoto struct, related data structs and methods.
// The Nakamoto leverages lib_chain, lib_miner, lib_tx_pool and lib_network to implement the Nakamoto consensus algorithm.
// You can see detailed instructions in the comments below.

use lib_chain::block::{
    BlockNode, BlockNodeHeader, BlockTree, MerkleTree, Puzzle, Transaction, Transactions,
};
use lib_miner::miner::{Miner, PuzzleSolution};
use lib_network::netchannel::NetAddress;
use lib_network::p2pnetwork::P2PNetwork;
use lib_tx_pool::pool::TxPool;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, RwLock};
use std::{thread, time::Duration};

type UserId = String;

/// The struct to represent configuration of the Nakamoto instance.
/// The configuration does not contain any user information. The Nakamoto algorithm is user-independent.
/// The configuration sets information about neighboring nodes, miner, block creation, etc.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    /// the list of addresses of neighboring nodes
    pub neighbors: Vec<NetAddress>,
    /// the address of this node
    pub addr: NetAddress,
    /// the number of threads used to mine a new block (for miner)
    pub miner_thread_count: u16,
    /// the length of the nonce string (for miner)
    pub nonce_len: u16,
    // difficulty to mine a new block (for miner)
    pub difficulty_leading_zero_len: u16,
    // difficulty to accept a new block (for verifying the block)
    pub difficulty_leading_zero_len_acc: u16,
    // the seed for the miner thread 0 (for miner)
    pub miner_thread_0_seed: u64,
    // the reward receiver (for mined blocks)
    pub mining_reward_receiver: UserId,
    // the max number of transactions in one block (for creating a new block)
    pub max_tx_in_one_block: u16,
}

/// Create a puzzle for the miner given a chain and a tx pool (as smart pointers).
/// It returns the puzzle (serialization of the Puzzle struct) and the corresponding incomplete block (nonce and block_id not filled)
fn create_puzzle(
    chain_p: Arc<Mutex<BlockTree>>,
    tx_pool_p: Arc<Mutex<TxPool>>,
    tx_count: u16,
    reward_receiver: UserId,
) -> (String, BlockNode) {
    // Please fill in the blank
    // Filter transactions from tx_pool and get the last node of the longest chain.
    // todo();

    let blocktree = chain_p.lock().unwrap();
    let txpool = tx_pool_p.lock().unwrap();

    let finalized_txs = &blocktree.finalized_tx_ids;
    let mut excluding_txs = Vec::<Transaction>::new();
    // excluding txs are txs that are not in finalized_txs
    for tx in txpool.pool_tx_ids.iter() {
        if !finalized_txs.contains(tx) {
            let txs = txpool.pool_tx_map.get(tx).unwrap().clone();
            excluding_txs.push(txs);
        }
    }
    let filtered_txs = txpool.filter_tx(tx_count, &excluding_txs);
    let last_block_id = blocktree.working_block_id.clone();
    let last_block = blocktree.get_block(last_block_id).unwrap().clone();

    // // build the puzzle
    let puzzle = Puzzle {
        // Please fill in the blank
        // Create a puzzle with the block_id of the parent node and the merkle root of the transactions.
        parent: last_block.header.parent.clone(),
        merkle_root: last_block.header.merkle_root.clone(),
        reward_receiver: reward_receiver.clone(),
    };
    let puzzle_str = serde_json::to_string(&puzzle).unwrap().to_owned();

    // Please fill in the blank
    // Create a block node with the transactions and the merkle root.
    // Leave the nonce and the block_id empty (to be filled after solving the puzzle).
    // The timestamp can be set to any positive interger.
    // In the end, it returns  (puzzle_str, pre_block);

    let pre_block = BlockNode {
        header: BlockNodeHeader {
            parent: last_block.header.parent.clone(),
            merkle_root: last_block.header.merkle_root.clone(),
            reward_receiver: reward_receiver.clone(),
            nonce: "".to_string(),
            block_id: "".to_string(),
            timestamp: 1,
        },

        transactions_block: Transactions {
            transactions: filtered_txs.clone(),
            merkle_tree: MerkleTree::create_merkle_tree(filtered_txs.clone()).1,
        },
    };

    return (puzzle_str, pre_block);
}

/// The struct to represent the Nakamoto instance.
/// The Nakamoto instance contains the chain, the miner, the network and the tx pool as smart pointers.
/// It also contains a FIFO channel for sending transactions to the Blockchain
pub struct Nakamoto {
    /// the chain (BlockTree)
    pub chain_p: Arc<Mutex<BlockTree>>,
    /// the miner
    pub miner_p: Arc<Mutex<Miner>>,
    /// the p2pnetwork
    pub network_p: Arc<Mutex<P2PNetwork>>,
    /// the transaction pool
    pub tx_pool_p: Arc<Mutex<TxPool>>,
    /// the FIFO channel for sending transactions to the Blockchain
    trans_tx: Sender<Transaction>,
}

impl Nakamoto {
    /// A function to send notification messages to stdout (For debugging purpose only)
    pub fn stdout_notify(msg: String) {
        let msg = HashMap::from([("Notify".to_string(), msg.clone())]);
        println!("{}", serde_json::to_string(&msg).unwrap());
    }

    /// Create a Nakamoto instance given the serialized chain, tx pool and config as three json strings.
    pub fn create_nakamoto(chain_str: String, tx_pool_str: String, config_str: String) -> Nakamoto {
        // Please fill in the blank
        // Deserialize the config from the given json string.
        // Deserialize the chain and the tx pool from the given json strings.
        // Create the miner and the network according to the config.
        // Start necessary threads that read from and write to FIFO channels provided by the network.
        // Start necessary thread(s) to control the miner.
        // Return the Nakamoto instance that holds pointers to the chain, the miner, the network and the tx pool.

        // Deserialize the config from the given json string.
        let config: Config =
            serde_json::from_str(&config_str).expect("Failed to deserialize config");
        let chain: Arc<Mutex<BlockTree>> = Arc::new(Mutex::new(
            serde_json::from_str(&chain_str).expect("Failed to deserialize chain"),
        ));
        let tx_pool: Arc<Mutex<TxPool>> = Arc::new(Mutex::new(
            serde_json::from_str(&tx_pool_str).expect("Failed to deserialize tx pool"),
        ));

        // Create the miner and the network according to the config.
        let miner = Miner {
            thread_count: 0,
            leading_zero_len: 0,
            is_running: false,
        };
        let arc_miner = Arc::new(Mutex::new(miner));
        let network = P2PNetwork::create(config.addr, config.neighbors);

        // Start necessary threads that read from and write to FIFO channels provided by the network.
        // Start necessary thread(s) to control the miner.
        //todo

        // Return the Nakamoto instance that holds pointers to the chain, the miner, the network and the tx pool.
        Nakamoto {
            chain_p: chain,
            miner_p: arc_miner,
            network_p: network.0,
            tx_pool_p: tx_pool,
            trans_tx: network.4,
        }
    }

    /// Get the status of the network as a dictionary of strings. For debugging purpose.
    pub fn get_network_status(&self) -> BTreeMap<String, String> {
        self.network_p.lock().unwrap().get_status()
    }

    /// Get the status of the chain as a dictionary of strings. For debugging purpose.
    pub fn get_chain_status(&self) -> BTreeMap<String, String> {
        self.chain_p.lock().unwrap().get_status()
    }

    /// Get the status of the transaction pool as a dictionary of strings. For debugging purpose.
    pub fn get_txpool_status(&self) -> BTreeMap<String, String> {
        self.tx_pool_p.lock().unwrap().get_status()
    }

    /// Get the status of the miner as a dictionary of strings. For debugging purpose.
    pub fn get_miner_status(&self) -> BTreeMap<String, String> {
        self.miner_p.lock().unwrap().get_status()
    }

    /// Publish a transaction to the Blockchain
    pub fn publish_tx(&mut self, transaction: Transaction) -> () {
        // Please fill in the blank
        // Add the transaction to the transaction pool and send it to the broadcast channel

        let mut tx_pool = self.tx_pool_p.lock().unwrap();
        tx_pool.add_tx(transaction.clone());
    }

    /// Get the serialized chain as a json string.
    pub fn get_serialized_chain(&self) -> String {
        let chain = self.chain_p.lock().unwrap().clone();
        serde_json::to_string_pretty(&chain).unwrap()
    }

    /// Get the serialized transaction pool as a json string.
    pub fn get_serialized_txpool(&self) -> String {
        let tx_pool = self.tx_pool_p.lock().unwrap().clone();
        serde_json::to_string_pretty(&tx_pool).unwrap()
    }
}
