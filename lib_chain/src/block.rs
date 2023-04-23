// This file is part of the project for the module CS3235 by Prateek
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

use base64ct::{Base64, Encoding};
/// This file contains the definition of the BlockTree
/// The BlockTree is a data structure that stores all the blocks that have been mined by this node or received from other nodes.
/// The longest path in the BlockTree is the main chain. It is the chain from the root to the working_block_id.
use core::panic;
use serde::{Deserialize, Serialize};
use sha2::{digest::block_buffer::Block, Digest, Sha256};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    convert,
    error::Error,
    hash,
    sync::Arc,
};

use pem::parse;
use rsa::pkcs1::{
    DecodeRsaPrivateKey, DecodeRsaPublicKey, EncodeRsaPrivateKey, EncodeRsaPublicKey,
};
use rsa::pkcs1v15::{SigningKey, VerifyingKey};
use rsa::signature::{RandomizedSigner, Signature as RSASig, Signer, Verifier};
use rsa::{RsaPrivateKey, RsaPublicKey};
pub type UserId = String;
pub type BlockId = String;
pub type Signature = String;
pub type TxId = String;

/// Merkle tree is used to verify the integrity of transactions in a block.
/// It is generated from a list of transactions. It will be stored inside `Transactions` struct.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MerkleTree {
    /// A list of lists of hashes, where the first list is the list of hashes of the transactions,
    /// the second list is the list of hashes of the first list, and so on.
    /// See the `create_merkle_tree` function for more details.
    pub hashes: Vec<Vec<String>>,
}

impl MerkleTree {
    /// Create a merkle tree from a list of transactions.
    /// The merkle tree is a list of lists of hashes,
    /// where the first list is the list of hashes of the transactions.
    /// The last list is the list with only one hash, called the Merkle root.
    /// - `txs`: a list of transactions
    /// - The return value is the root hash of the merkle tree
    pub fn create_merkle_tree(txs: Vec<Transaction>) -> (String, MerkleTree) {
        if txs.is_empty() {
            panic!("create_merkle_tree received empty transaction vector.");
        }
        // todo!()

        // To create a Merkle tree from a list of transactions, you can follow these steps:
        // Create a list of hashes of all transactions.
        // If the number of hashes is odd, duplicate the last hash to make it even.
        // Group hashes into pairs and hash each pair to get a new list of hashes.
        // If the number of hashes is still not one, repeat steps 2 and 3 until you get a single hash, which is the Merkle root.

        let mut hashes: Vec<Vec<String>> = vec![txs.iter().map(|tx| tx.gen_hash()).collect()];

        while hashes.last().unwrap().len() > 1 {
            let mut level: Vec<String> = Vec::new();
            let last_level = hashes.last().unwrap();

            if last_level.len() % 2 != 0 {
                let last_hash = last_level.last().unwrap().to_string();
                level.push(last_hash.clone());
            }

            for i in (0..last_level.len() - 1).step_by(2) {
                let mut hasher = Sha256::new();

                let h1 = &last_level[i];
                let h2 = &last_level[i + 1];

                let mut owned_string: String = h1.to_owned();
                owned_string.push_str(&h2);
                let input = owned_string.as_bytes();

                hasher.update(input);
                let result = hasher.finalize();

                level.push(hex::encode(result));
            }

            hashes.push(level);
        }

        let root = hashes.last().unwrap()[0].clone();
        let tree = MerkleTree { hashes };

        (root, tree)
    }
}

/// The struct containing a list of transactions and the merkle tree of the transactions.
/// Each block will contain one `Transactions` struct.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Transactions {
    /// The merkle tree of the transactions
    pub merkle_tree: MerkleTree,
    /// A list of transactions
    pub transactions: Vec<Transaction>,
}

/// The struct is used to store the information of one transaction.
/// The transaction id is not stored explicitly, but can be generated from the transaction using the `gen_hash` function.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Transaction {
    /// The user_id of the sender
    pub sender: UserId,
    /// The user_id of the receiver
    pub receiver: UserId,
    /// The message of the transaction.
    /// The expected format is `SEND $300   // By Alice   // 1678173972743`,
    /// where `300` is the amount of money to be sent,
    /// and the part after the first `//` is the comment: `Alice` is the friendly name of the sender, and `1678173972743` is the timestamp of the transaction.
    /// The comment part does not affect the validity of the transaction nor the computation of the balance.
    pub message: String,
    /// The signature of the transaction in base64 format
    pub sig: Signature,
}

impl Transaction {
    /// Create a new transaction struct given the sender, receiver, message, and signature.
    pub fn new(sender: UserId, receiver: UserId, message: String, sig: Signature) -> Transaction {
        Transaction {
            sender,
            receiver,
            message,
            sig,
        }
    }

    /// Compute the transaction id from the transaction. The transaction id is the sha256 hash of the serialized transaction struct in hex format.
    pub fn gen_hash(&self) -> TxId {
        let mut hasher = Sha256::new();
        let hasher_str = serde_json::to_string(&self).unwrap();
        hasher.update(hasher_str);
        let result = hasher.finalize();
        let tx_hash: TxId = format!("{:x}", result);
        tx_hash
    }

    /// Verify the signature of the transaction. Return true if the signature is valid, and false otherwise.
    pub fn verify_sig(&self) -> bool {
        // Please fill in the blank
        // verify the signature using the sender_id as the public key (you might need to change the format into PEM)
        // You can look at the `verify` function in `bin_wallet` for reference. They should have the same functionality.
        // todo!();

        // All lines except the last line must be 64 characters in length ...haizz
        let formatted_string = format!(
            "{}{}",
            &self.sender[..64],
            "\n".to_string() + &self.sender[64..]
        );

        // convert the public key into PEM format
        let pem_encoded_key = format!(
            "-----BEGIN RSA PUBLIC KEY-----\n{}\n-----END RSA PUBLIC KEY-----\n",
            formatted_string
        );

        let public_key = rsa::RsaPublicKey::from_pkcs1_pem(&pem_encoded_key).unwrap();
        let verifying_key = VerifyingKey::<Sha256>::new(public_key);
        let signature = Base64::decode_vec(&self.sig).unwrap();
        let verify_signature = RSASig::from_bytes(&signature).unwrap();

        // message is a tuple (sender, receiver, message) serialized to a string
        let mut msg: String = "[\"".to_string();
        msg.push_str(&self.sender);
        msg.push_str("\",\"");
        msg.push_str(&self.receiver);
        msg.push_str("\",\"");
        msg.push_str(&self.message);
        msg.push_str("\"]");

        let verify_result = verifying_key.verify(msg.as_bytes(), &verify_signature);

        return match verify_result {
            Ok(()) => true,
            Err(e) => {
                println!("[Signature verification failed]: {}", e);
                false
            }
        };
    }
}

/// The struct representing a whole block tree.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockTree {
    /// A map from block id to the block node
    pub all_blocks: HashMap<BlockId, BlockNode>,
    /// A map from block id to the list of its children (as block ids)
    pub children_map: HashMap<BlockId, Vec<BlockId>>,
    /// A map from block id to the depth of the block. The genesis block has depth 0.
    pub block_depth: HashMap<BlockId, u64>,
    /// The id of the root block (the genesis block)
    pub root_id: BlockId,
    /// The id of the working block (the block at the end of the longest chain)
    pub working_block_id: BlockId,
    /// A map to bookkeep the orphan blocks.
    /// Orphan blocks are blocks whose parent are not in the block tree yet.
    /// They should be added to the block tree once they can be connected to the block tree.
    pub orphans: HashMap<BlockId, BlockNode>,
    /// The id of the latest finalized block
    pub finalized_block_id: BlockId,
    /// A map from the user id to its balance
    pub finalized_balance_map: HashMap<UserId, i64>,
    /// A set of transaction ids that have been finalized. It includes all the transaction ids in the finalized blocks.
    pub finalized_tx_ids: HashSet<TxId>,
}

impl BlockTree {
    /// Create a new block tree with the genesis block as the root.
    pub fn new() -> BlockTree {
        let mut bt = BlockTree {
            all_blocks: HashMap::new(),
            children_map: HashMap::new(),
            block_depth: HashMap::new(),
            root_id: String::new(),
            working_block_id: String::new(),
            orphans: HashMap::new(),
            finalized_block_id: String::new(),
            finalized_balance_map: HashMap::new(),
            finalized_tx_ids: HashSet::new(),
        };
        let genesis_block = BlockNode::genesis_block();
        bt.all_blocks.insert("0".to_string(), genesis_block.clone());
        bt.block_depth.insert("0".to_string(), 0);
        bt.root_id = "0".to_string();
        bt.working_block_id = "0".to_string();
        for tx in genesis_block.transactions_block.transactions {
            let amount = tx.message.split(" ").collect::<Vec<&str>>()[1]
                .trim_start_matches('$')
                .parse::<i64>()
                .unwrap();
            bt.finalized_balance_map.insert(tx.receiver, amount);
        }
        bt.finalized_block_id = "0".to_string();
        bt
    }

    /// Add a block to the block tree. If the block is not valid to be added to the tree
    /// (i.e. it does not satsify the conditions below), ignore the block. Otherwise, add the block to the BlockTree.
    ///
    /// 1. The block must have a valid nonce and the hash in the puzzle solution satisfies the difficulty requirement. done
    /// 2. The block_id of the block must be equal to the computed hash in the puzzle solution. done
    /// 3. The block does not exist in the block tree or the orphan map. done
    /// 4. The transactions in the block must be valid. See the `verify_sig` function in the `Transaction` struct for details. done
    /// 5. The parent of the block must exist in the block tree.
    ///     Otherwise, it will be bookkeeped in the orphans map.
    ///     When the parent block is added to the block tree, the block will be removed from the orphan map and checked against the conditions again. done
    /// 6. The transactions in the block must not be duplicated with any transactions in its ancestor blocks. done
    /// 7. Each sender in the txs in the block must have enough balance to pay for the transaction. done
    ///    Conceptually, the balance of one address is the sum of the money sent to the address minus the money sent from the address
    ///    when walking from the genesis block to this block, according to the order of the txs in the blocks. done
    ///    Mining reward is a constant of $10 (added to the reward_receiver address **AFTER** considering transactions in the block). done
    ///
    /// When a block is successfully added to the block tree, update the related fields in the BlockTree struct
    /// (e.g., working_block_id, finalized_block_id, finalized_balance_map, finalized_tx_ids, block_depth, children_map, all_blocks, etc)

    pub fn add_block(&mut self, block: BlockNode, leading_zero_len: u16) -> Result<(), String> {
        //     todo!();

        let block_id = block.header.block_id.clone();
        let parent_id = block.header.parent.clone();

        // Ensure that the block does not exist in the block tree or the orphan map.
        if self.all_blocks.contains_key(&block_id) || self.orphans.contains_key(&block_id) {
            return Err("Block already exists in the block tree or orphan map.".to_string());
        }

        // Ensure that block is valid
        if (&block).validate_block(leading_zero_len) != (true, block_id.clone()) {
            return Err("Block is not valid.".to_string());
        }

        // Verify that the parent of the block exists in the block tree, otherwise, add it to the orphans map.
        let _parent_node = match self.all_blocks.get(&parent_id) {
            Some(parent_node) => parent_node,
            None => {
                self.orphans.insert(block_id.clone(), block);
                return Ok(()); // Return early since block is in the orphan map
            }
        };

        self.all_blocks.insert(block_id.clone(), block.clone());
        self.block_depth.insert(
            block_id.clone(),
            self.block_depth.get(&parent_id).unwrap() + 1,
        );

        // Add block to parent's children list
        let children = self
            .children_map
            .entry(parent_id.clone())
            .or_insert_with(Vec::new);
        children.push(block_id.clone());
        let block_node = self.all_blocks.get_mut(&block_id).unwrap();
        block_node.header.parent = parent_id.clone();

        // Update block depth
        let parent_depth = self.block_depth.get(&parent_id).unwrap();
        let block_depth = parent_depth + 1;
        self.block_depth.insert(block_id.clone(), block_depth);

        // Add any orphans that have this block as a parent
        let mut orphans_to_add = Vec::new();
        for (orphan_id, orphan_block) in self.orphans.iter() {
            if orphan_block.header.parent == block_id {
                orphans_to_add.push(orphan_id.clone());
            }
        }
        for orphan_id in orphans_to_add {
            let orphan_block = self.orphans.remove(&orphan_id).unwrap();
            self.add_block(orphan_block, leading_zero_len)?;
        }

        // Update longest path (working_block_id)
        if self.block_depth.get(&block_id).unwrap()
            > self.block_depth.get(&self.working_block_id).unwrap()
        {
            self.working_block_id = block_id.clone();
        } else if self.block_depth.get(&self.working_block_id).unwrap()
            == self.block_depth.get(&block_id).unwrap()
        {
            // the one whose last block has the larger hash number as the longest path
            if block_id > self.working_block_id {
                self.working_block_id = block_id.clone();
            }
        }

        let txs = self.get_pending_finalization_txs();
        let txss = self.get_pending_finalization_txs();

        // Verify that each sender in the transactions in the block has enough balance to pay for the transaction.
        let mut balance_map = self.finalized_balance_map.clone();

        // Transfer money from sender to receiver
        for tx in txs {
            let sender = &tx.sender;
            let receiver = &tx.receiver;
            let message = &tx.message;
            let amount_str = message
                .split("$")
                .nth(1)
                .unwrap()
                .split(" ")
                .next()
                .unwrap();
            let amount = amount_str.parse::<i64>().unwrap();

            if !balance_map.contains_key(sender) || balance_map[sender] < amount {
                return Err(format!(
                    "Sender {} does not have enough balance to pay for transaction.",
                    sender
                ));
            }
            balance_map
                .entry(sender.clone())
                .and_modify(|e| *e -= amount);

            // Check if receiver exists in balance map, if not, add it
            if !balance_map.contains_key(receiver) {
                balance_map.insert(receiver.clone(), amount);
            } else {
                balance_map
                    .entry(receiver.clone())
                    .and_modify(|e| *e += amount);
            }
        }

        // self.working_block_id = block_id.clone();
        // self.all_blocks.insert(block_id.clone(), block.clone());

        // Update finalized tx ids
        let mut temp = self.finalized_tx_ids.clone();
        for tx in txss {
            temp.insert(tx.gen_hash());
        }
        self.finalized_tx_ids = temp;

        let finalized_blocks = self.get_finalized_blocks_since(self.finalized_block_id.clone());
        if !finalized_blocks.is_empty() {
            self.finalized_block_id = finalized_blocks[0].header.block_id.clone();
            // Add $10 to reward receiver; if reward receiver does not exist in balance map, add it
            let block = &finalized_blocks[0];
            if balance_map.contains_key(&block.header.reward_receiver) {
                balance_map
                    .entry(block.header.reward_receiver.clone())
                    .and_modify(|e| *e += 10);
            } else {
                balance_map.insert(block.header.reward_receiver.clone(), 10);
            }
        }

        // Update balance map
        self.finalized_balance_map = balance_map;

        Ok(())
    }

    /// Get the block node by the block id if exists. Otherwise, return None.
    pub fn get_block(&self, block_id: BlockId) -> Option<BlockNode> {
        // Please fill in the blank
        // todo!();
        for (_, block) in self.all_blocks.iter() {
            if block.header.block_id == block_id {
                return Some(block.clone());
            }
        }
        return None;
    }

    /// Get the finalized blocks on the longest path after the given block id, from the oldest to the most recent.
    /// The given block id should be any of the ancestors of the current finalized block id or the current finalized block id itself.
    /// If it is not the case, the function will panic (i.e. we do not consider inconsistent block tree caused by attacks in this project)
    pub fn get_finalized_blocks_since(&self, since_block_id: BlockId) -> Vec<BlockNode> {
        // Please fill in the blank
        // todo!();

        let mut finalized_blocks = Vec::new();
        let mut block_id = self.working_block_id.clone();
        let depth = self.block_depth[&block_id];
        while block_id != since_block_id {
            let id = block_id.clone();
            let block = self.get_block(block_id).unwrap();
            if (depth - self.block_depth[&id]) >= 6 {
                finalized_blocks.push(block.clone());
            }
            block_id = block.header.parent;
        }
        finalized_blocks.reverse(); // oldest to newest
        return finalized_blocks;
    }

    /// Get the pending transactions on the longest chain that are confirmed but not finalized.
    pub fn get_pending_finalization_txs(&self) -> Vec<Transaction> {
        // Please fill in the blank
        // todo!();
        let mut pending_txs = Vec::new();
        let blocks = self.get_finalized_blocks_since(self.finalized_block_id.clone());
        for block in blocks {
            for tx in block.transactions_block.transactions.iter() {
                if !self.finalized_tx_ids.contains(&tx.gen_hash()) {
                    pending_txs.push(tx.clone());
                }
            }
        }
        return pending_txs;
    }

    /// Get status information of the BlockTree for debug printing.
    pub fn get_status(&self) -> BTreeMap<String, String> {
        // Please fill in the blank
        // For debugging purpose, you can return any dictionary of strings as the status of the BlockTree.
        // It should be displayed in the Client UI eventually.
        // todo!();
        let mut status = BTreeMap::new();
        status.insert("#blocks".to_string(), self.all_blocks.len().to_string());
        status.insert("#orphans".to_string(), self.orphans.len().to_string());
        status.insert(
            "finalized_id".to_string(),
            self.finalized_block_id.len().to_string(),
        );
        status.insert("root_id".to_string(), self.root_id.len().to_string());
        status.insert(
            "working_depth".to_string(),
            self.block_depth[&self.working_block_id].to_string(),
        );
        status.insert(
            "working_id".to_string(),
            self.working_block_id.len().to_string(),
        );

        status
    }
}

/// The struct representing a puzzle for the miner to solve. The puzzle is to find a nonce such that when concatenated
/// with the serialized json string of this `Puzzle` struct, the sha256 hash of the result has the required leading zero length.
#[derive(Serialize)]
pub struct Puzzle {
    pub parent: BlockId,
    pub merkle_root: String,
    pub reward_receiver: UserId,
}

/// The struct representing a block header. Each `BlockNode` has one `BlockNodeHeader`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BlockNodeHeader {
    /// The block id of the parent block.
    pub parent: BlockId,
    /// The merkle root of the transactions in the block.
    pub merkle_root: String,
    /// The timestamp of the block. For genesis block, it is 0. For other blocks, greater or equal to 1 is considered valid.
    pub timestamp: u64,
    /// The block id of the block (the block id is the sha256 hash of the concatination of the nonce and a `Puzzle` derived from the block)
    pub block_id: BlockId,
    /// The nonce is the solution found by the miner for the `Puzzle` derived from this block.
    pub nonce: String,
    /// The reward receiver of the block.
    pub reward_receiver: UserId,
}

/// The struct representing a block node.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BlockNode {
    /// The header of the block.
    pub header: BlockNodeHeader,
    /// The transactions in the block.
    pub transactions_block: Transactions,
}

impl BlockNode {
    /// Create the genesis block that contains the initial transactions
    /// (give $299792458 to the address of Alice `MDgCMQCqrJ1yIJ7cDQIdTuS+4CkKn/tQPN7bZFbbGCBhvjQxs71f6Vu+sD9eh8JGpfiZSckCAwEAAQ==`)
    pub fn genesis_block() -> BlockNode {
        let header = BlockNodeHeader {
            parent: "0".to_string(),
            merkle_root: "0".to_string(),
            timestamp: 0,
            block_id: "0".to_string(),
            nonce: "0".to_string(),
            reward_receiver: "GENESIS".to_string(),
        };

        let transactions_block = Transactions {
            transactions: vec![Transaction::new(
                "GENESIS".to_owned(),
                "MDgCMQCqrJ1yIJ7cDQIdTuS+4CkKn/tQPN7bZFbbGCBhvjQxs71f6Vu+sD9eh8JGpfiZSckCAwEAAQ=="
                    .to_string(),
                "SEND $299792458".to_owned(),
                "GENESIS".to_owned(),
            )],
            merkle_tree: MerkleTree { hashes: vec![] }, // Skip merkle tree generation for genesis block
        };

        BlockNode {
            header,
            transactions_block,
        }
    }

    /// Check for block validity based solely on this block (not considering its validity inside a block tree).
    /// Return a tuple of (bool, String) where the bool is true if the block is valid and false otherwise.
    /// The string is the re-computed block id.
    /// The following need to be checked:
    /// 1. The block_id in the block header is indeed the sha256 hash of the concatenation of the nonce and the serialized json string of the `Puzzle` struct derived from the block.
    /// 2. All the transactions in the block are valid.
    /// 3. The merkle root in the block header is indeed the merkle root of the transactions in the block.
    pub fn validate_block(&self, leading_zero_len: u16) -> (bool, BlockId) {
        // Please fill in the blank
        // todo!();

        let mut hasher = Sha256::new();
        let block_nonce = self.header.nonce.clone();
        let block_id = self.header.block_id.clone();

        // Check that the block's hash satisfies the difficulty requirement.
        if !block_id.starts_with(&"0".repeat(leading_zero_len as usize)) {
            println!("Block does not satisfy difficulty requirement.");
            return (false, block_id);
        }

        // Create a puzzle struct from the block header and serialize it to a json string.
        let puzzle = Puzzle {
            parent: self.header.parent.clone(),
            merkle_root: self.header.merkle_root.clone(),
            reward_receiver: self.header.reward_receiver.clone(),
        };
        let serialized = serde_json::to_string(&puzzle).unwrap().to_owned();

        let mut owned_string: String = block_nonce.clone();
        owned_string.push_str(&serialized);
        hasher.update(owned_string.as_bytes());
        let res = hasher.finalize();

        // Verify that the block_id of the block is equal to the computed hash in the puzzle solution.
        if hex::encode(res) != block_id {
            println!(
                "Block ID does not match computed hash in puzzle solution.{} {}",
                block_id,
                hex::encode(res)
            );
            return (false, hex::encode(res));
        }

        // Verify that the transactions in the block are valid using the `verify_sig` function in the `Transaction` struct.
        let verified = self
            .transactions_block
            .transactions
            .iter()
            .all(|tx| tx.verify_sig());
        if !verified {
            println!("Block contains invalid transactions.");
            return (false, block_id);
        }

        // Verify merkle root of the block matches the merkle root of transactions.
        let root = self.transactions_block.merkle_tree.hashes.last().unwrap()[0].clone();
        if root != self.header.merkle_root {
            println!("Block merkle root does not match merkle root of transactions.");
            return (false, block_id);
        }
        return (true, block_id);
    }
}
