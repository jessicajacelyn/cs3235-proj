// This file is part of the project for the module CS3235 by Prateek
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

/// This is the main file of the bin_nakamoto executable.
/// It is a simple command-line program that can be used to interact with the Blockchain
/// It reads commands from stdin and writes responses to stdout to facilitate IPC communication with bin_client eventually.
/// However, you can also run it directly from the command line to test it.
/// You can see detailed instructions in the comments below.
mod nakamoto;
use lib_chain::block::{BlockTree, Signature, Transaction};
use nakamoto::Nakamoto;

use seccompiler::BpfMap;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, BufRead, Write};

// Read a string from a file (to help you debug)
fn read_string_from_file(filepath: &str) -> String {
    let contents = fs::read_to_string(filepath).expect(&("Cannot read ".to_owned() + filepath));
    contents
}

// Append a string to a file (to help you debug)
fn append_string_to_file(filepath: &str, content: String) {
    // if not exists, create file
    if !std::path::Path::new(filepath).exists() {
        fs::File::create(filepath).unwrap();
    }
    fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open(filepath)
        .unwrap()
        .write_all(content.as_bytes())
        .unwrap();
}

/// This enum represents IPC messsage requests from the stdin
#[derive(Serialize, Deserialize, Debug, Clone)]
enum IPCMessageReq {
    /// Initialize the Nakamoto instance using the given (blocktree_json, tx_pool_json, config_json)
    Initialize(String, String, String),
    /// Get the balance of the given address (user_id)
    GetAddressBalance(String),
    /// Publish a transaction to the network (data_string, signature)
    PublishTx(String, Signature),
    /// Get the block data of the given block_id
    RequestBlock(String),
    /// Get the network status (for debugging)
    RequestNetStatus,
    /// Get the chain status (for debugging)
    RequestChainStatus,
    /// Get the miner status (for debugging)
    RequestMinerStatus,
    /// Get the tx pool status (for debugging)
    RequestTxPoolStatus,
    /// Get the state serialization (including BlockTree and TxPool)
    RequestStateSerialization,
    /// Quit the program
    Quit,
}

/// This enum represents IPC messsage responses to the stdout
#[derive(Serialize, Deserialize, Debug, Clone)]
enum IPCMessageResp {
    /// The Nakamoto instance has been initialized (responding to Initialize)
    Initialized,
    /// The transaction has been published (responding to PublishTx)
    PublishTxDone,
    /// The balance of the given address (user_id, balance)
    AddressBalance(String, i64),
    /// The block data of the given block_id (block_data)
    BlockData(String),
    /// The network status as a dictionary of strings (for debugging)
    NetStatus(BTreeMap<String, String>),
    /// The chain status as a dictionary of strings (for debugging)
    ChainStatus(BTreeMap<String, String>),
    /// The miner status as a dictionary of strings (for debugging)
    MinerStatus(BTreeMap<String, String>),
    /// The tx pool status as a dictionary of strings (for debugging)
    TxPoolStatus(BTreeMap<String, String>),
    /// The state serialization (blocktree_json_string, tx_pool_json_string)
    StateSerialization(String, String),
    /// The program is quitting (responding to Quit)
    Quitting,
    /// This is not an actual response, but an arbitrary notification message for debugging
    Notify(String),
}

fn main() {
    // bin_nakamoto has only one optional argument: the path to the seccomp policy file
    // If the argument is provided, bin_nakamoto will read and apply the seccomp policy at the beginning of the program
    // Otherwise, it will proceed to the normal execution
    let maybe_policy_path = std::env::args().nth(1);
    if let Some(policy_path) = maybe_policy_path {
        // Please fill in the blank
        // If the first param is provided, read the seccomp config and apply it
        let filter_map: BpfMap = seccompiler::compile_from_json(
            read_string_from_file(&policy_path).as_bytes(),
            std::env::consts::ARCH.try_into().unwrap(),
        )
        .unwrap();
        let filter = filter_map.get("main_thread").unwrap();

        seccompiler::apply_filter(&filter).unwrap();
    }

    // The main logic of the bin_nakamoto starts here
    // It reads IPC calls from stdin and write IPC responses to stdout in a loop.
    // The first IPC call should be Initialize, whose parameters are serialized BlockTree, TxPool, and Config.
    // After that, there can be artitrary number of IPC calls, including GetAddressBalance, PublishTx, RequestBlock, RequestNetStatus, RequestChainStatus, RequestMinerStatus, RequestTxPoolStatus, RequestStateSerialization, etc.
    // Eventually, the program will quit when receiving a Quit IPC call.
    // Please fill in the blank
    // Loop over stdin and handle IPC messages
    let mut nakamoto: Option<Nakamoto> = None;
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let input = line.unwrap();
        let req: IPCMessageReq =
            serde_json::from_str(&input).expect("Failed to parse input as IPCMessageReq");
        let response = match req {
            IPCMessageReq::Initialize(blocktree_json, tx_pool_json, config_json) => {
                // Initialize the Nakamoto instance using the given (blocktree_json, tx_pool_json, config_json)
                nakamoto = Some(Nakamoto::create_nakamoto(
                    blocktree_json,
                    tx_pool_json,
                    config_json,
                ));

                IPCMessageResp::Initialized
            }
            IPCMessageReq::GetAddressBalance(user_id) => {
                let nakamoto = nakamoto
                    .as_ref()
                    .expect("Nakamoto instance not initialized");
                let chain = nakamoto.get_serialized_chain();
                // Deserialize the chain
                let deserialized_chain: BlockTree = serde_json::from_str(&chain).unwrap();
                // Get the balance of the given address
                let balance = deserialized_chain
                    .finalized_balance_map
                    .get(&user_id)
                    .unwrap();

                IPCMessageResp::AddressBalance(user_id, *balance)
            }
            IPCMessageReq::PublishTx(data_string, signature) => {
                // Publish a transaction to the network (data_string, signature)

                // Get sender from data_string
                // Remove the first and last three characters
                let data_string = data_string[3..data_string.len() - 3].to_string();
                let split_data_string = data_string.split("\",\"").collect::<Vec<&str>>();
                let sender_id = split_data_string[0].to_string();
                let receiver_id = split_data_string[1].to_string();
                let msg = split_data_string[2].to_string();

                // Create a transaction instance
                let tx = Transaction {
                    sender: sender_id,
                    receiver: receiver_id,
                    message: msg,
                    sig: signature,
                };
                // Publish to the network
                let nakamoto = nakamoto.as_mut().unwrap();
                nakamoto.publish_tx(tx);

                IPCMessageResp::PublishTxDone
            }
            IPCMessageReq::RequestBlock(block_id) => {
                let nakamoto = nakamoto
                    .as_ref()
                    .expect("Nakamoto instance not initialized");

                let chain = nakamoto.get_serialized_chain();
                // Deserialize the chain
                let deserialized_chain: BlockTree = serde_json::from_str(&chain).unwrap();
                // Get the block data of the given block_id and serialize it
                let block_data = deserialized_chain.all_blocks.get(&block_id).unwrap();
                let serialized_block_data = serde_json::to_string(&block_data).unwrap();

                //create block instance
                IPCMessageResp::BlockData(serialized_block_data)
            }
            IPCMessageReq::RequestNetStatus => {
                // Get the network status (for debugging)
                let nakamoto = nakamoto
                    .as_ref()
                    .expect("Nakamoto instance not initialized");
                IPCMessageResp::NetStatus(nakamoto.get_network_status())
            }
            IPCMessageReq::RequestChainStatus => {
                // Get the chain status (for debugging)
                let nakamoto = nakamoto
                    .as_ref()
                    .expect("Nakamoto instance not initialized");
                IPCMessageResp::ChainStatus(nakamoto.get_chain_status())
            }
            IPCMessageReq::RequestMinerStatus => {
                // Get the miner status (for debugging)
                let nakamoto = nakamoto
                    .as_ref()
                    .expect("Nakamoto instance not initialized");
                IPCMessageResp::MinerStatus(nakamoto.get_miner_status())
            }
            IPCMessageReq::RequestTxPoolStatus => {
                // Get the tx pool status (for debugging)
                let nakamoto = nakamoto
                    .as_ref()
                    .expect("Nakamoto instance not initialized");
                IPCMessageResp::TxPoolStatus(nakamoto.get_txpool_status())
            }
            IPCMessageReq::RequestStateSerialization => {
                // Get the state serialization (including BlockTree and TxPool)
                let nakamoto = nakamoto
                    .as_ref()
                    .expect("Nakamoto instance not initialized");
                IPCMessageResp::StateSerialization(
                    nakamoto.get_serialized_chain(),
                    nakamoto.get_serialized_txpool(),
                )
            }
            IPCMessageReq::Quit => {
                // Quit the program
                IPCMessageResp::Quitting
            }
        };
        let output = serde_json::to_string(&response).unwrap();
        println!("{}\n", output);
    }
}
