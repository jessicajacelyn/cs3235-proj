// This file is part of the project for the module CS3235 by Prateek
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

// This file contains the definition of the transaction pool.
// The transaction pool `TxPool` is a data structure that stores all the valid transactions that are not yet finalized.
// It helps with filtering the transactions that can be included in a new block.
use lib_chain::block::{BlockId, BlockNode, Signature, Transaction, TxId};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    convert,
    hash::Hash,
};

/// The maximum number of transactions that can be stored in the pool. Extra transactions will be dropped.
const MAX_TX_POOL: usize = 10000;

/// A transaction pool that stores received transactions that are not yet finalized.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TxPool {
    /// A list of transaction ids in the pool
    pub pool_tx_ids: Vec<TxId>,
    /// A map from transaction id (TxId) to transaction
    pub pool_tx_map: HashMap<TxId, Transaction>,
    /// A set of transaction ids that have been removed from the pool, so that duplicate transactions can be filtered out.
    pub removed_tx_ids: HashSet<TxId>,
    /// The id of the last finalized block. Transactions that are finalized will be removed from the pool and added to the removed_tx_ids set.
    pub last_finalized_block_id: BlockId,
}

impl TxPool {
    /// Create a new transaction pool
    pub fn new() -> TxPool {
        TxPool {
            pool_tx_ids: vec![],
            pool_tx_map: HashMap::new(),
            last_finalized_block_id: "0".to_string(),
            removed_tx_ids: HashSet::new(),
        }
    }

    /// Add a transaction `tx` to the pool if it satisfies the following conditions:
    /// - The transaction is not already in the pool
    /// - The transaction is not already in the removed_tx_ids set
    /// - The pool size is less than MAX_TX_POOL
    /// - The transaction has valid signature
    /// It returns true if the transaction satisfies the conditions above and is successfully added to the pool, and false otherwise.
    pub fn add_tx(&mut self, tx: Transaction) -> bool {
        // Please fill in the blank
        // todo!();

        // Check if the transaction is already in the pool or removed_tx_ids set
        if self.pool_tx_map.contains_key(&tx.gen_hash())
            || self.removed_tx_ids.contains(&tx.gen_hash())
        {
            return false;
        }

        // Check if the pool size is less than MAX_TX_POOL
        if self.pool_tx_ids.len() >= MAX_TX_POOL {
            return false;
        }

        // Check if the transaction has a valid signature
        if !tx.verify_sig() {
            return false;
        }

        // Add the transaction to the pool
        self.pool_tx_ids.push(tx.gen_hash());
        self.pool_tx_map.insert(tx.gen_hash(), tx);

        true
    }

    /// Deleting a tx from the pool. This function is used by remove_txs_from_finalized_blocks and some unit tests.
    /// It should update pool_tx_ids, pool_tx_map, and removed_tx_ids.
    /// If the transaction does not exist in the pool, make sure it is added to removed_tx_ids.
    pub fn del_tx(&mut self, tx_id: TxId) -> () {
        // Please fill in the blank
        // todo!();

        let id = tx_id.clone();
        // Check if the transaction exists in the pool
        if let Some(_transaction) = self.pool_tx_map.remove(&tx_id) {
            // Add the transaction ID to the set of removed transaction IDs
            self.removed_tx_ids.insert(tx_id);

            // Iterate over pool_tx_ids and remove the transaction ID
            let mut index = 0;
            while index < self.pool_tx_ids.len() {
                if self.pool_tx_ids[index] == id {
                    self.pool_tx_ids.remove(index);
                } else {
                    index += 1;
                }
            }
        } else {
            // If the transaction does not exist in the pool, add it to the set of removed transaction IDs
            self.removed_tx_ids.insert(tx_id);
        }
    }

    /// Filter `max_count` number of tx from the pool. It is used for creating puzzle.
    /// - `max_count`: the maximum number of transactions to be returned
    /// - `excluding_txs`: a list of transactions that should not be included in the returned list.
    ///                    It is used to filter out those transactions on the longest chain but hasn't been finalized yet.
    pub fn filter_tx(&self, max_count: u16, excluding_txs: &Vec<Transaction>) -> Vec<Transaction> {
        // Please fill in the blank
        // todo!();

        let mut filtered_txs: Vec<Transaction> = vec![];
        let mut count = 0;

        for tx_id in &self.pool_tx_ids {
            // Check if the transaction is not in the excluding_txs list
            if !excluding_txs.iter().any(|tx| &tx.gen_hash() == tx_id) {
                if let Some(tx) = self.pool_tx_map.get(tx_id) {
                    filtered_txs.push(tx.clone());
                    count += 1;
                    if count == max_count {
                        break;
                    }
                }
            }
        }

        filtered_txs
    }

    /// Remove transactions from the pool given a list of finalized blocks. Update last_finalized_block_id as the last block in the list.
    pub fn remove_txs_from_finalized_blocks(&mut self, finalized_blocks: &Vec<BlockNode>) {
        // Please fill in the blank
        // todo!();
        for block in finalized_blocks {
            for tx in &block.transactions_block.transactions {
                self.del_tx(tx.gen_hash());
            }
        }
        self.last_finalized_block_id = finalized_blocks.last().unwrap().header.block_id.clone();
    }

    /// Get status information of the tx_pool for debug printing.
    pub fn get_status(&self) -> BTreeMap<String, String> {
        // Please fill in the blank
        // For debugging purpose, you can return any dictionary of strings as the status of the tx_pool.
        // It should be displayed in the Client UI eventually.
        // todo!();
        let mut status = BTreeMap::new();
        status.insert(
            "pool_tx_map".to_string(),
            self.pool_tx_map.len().to_string(),
        );
        status
    }
}
