// This file is part of the project for the module CS3235 by Prateek 
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

/// P2PNetwork is a struct that implements a peer-to-peer network.
/// It is used to send and receive messages to/from neighbors.
/// It also automatically broadcasts messages. 
// You can see detailed instructions in the comments below.
// You can also look at the unit tests in ./lib.rs to understand the expected behavior of the P2PNetwork.


use lib_chain::block::{BlockNode, Transaction, BlockId, TxId};
use crate::netchannel::*;
use std::collections::{HashMap, BTreeMap, HashSet};
use std::io::BufReader;
use std::io::BufRead;
use std::{convert, io};
use std::net::{TcpListener, TcpStream, SocketAddr, ToSocketAddrs};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

/// The struct to represent statistics of a peer-to-peer network.
pub struct P2PNetwork {
    /// The number of messages sent by this node.
    pub send_msg_count: u64,
    /// The number of messages received by this node.
    pub recv_msg_count: u64,
    /// The address of this node.
    pub address: NetAddress,
    /// The addresses of the neighbors.
    pub neighbors: Vec<NetAddress>,
}

impl P2PNetwork {

    fn handle_incoming_connection(
        stream: TcpStream,
        block_node_tx: Sender<BlockNode>,
        transaction_tx: Sender<Transaction>,
        block_request_tx: Sender<BlockId>,
    ) {
        let peer_addr = stream.peer_addr().expect("Failed to get peer address");
        println!("Incoming connection from {}", peer_addr);
    
        // clone the senders to be moved into the thread
        let block_node_tx_clone = block_node_tx.clone();
        let transaction_tx_clone = transaction_tx.clone();
        let block_request_tx_clone = block_request_tx.clone();
    
        std::thread::spawn(move || {
            // create a buffered reader to read incoming messages from the stream
            let reader = BufReader::new(&stream);
    
            // read messages from the stream and handle them appropriately
            for line in reader.lines() {
                match line {
                    Ok(msg) => {
                        // parse the message and determine its type
                        if let Ok(block_node) = serde_json::from_str::<BlockNode>(&msg) {
                            // if the message is a BlockNode, send it to the block_node channel
                            if block_node_tx_clone.send(block_node).is_err() {
                                println!("Failed to send block node to channel");
                                break;
                            }
                        } else if let Ok(transaction) = serde_json::from_str::<Transaction>(&msg) {
                            // if the message is a Transaction, send it to the transaction channel
                            if transaction_tx_clone.send(transaction).is_err() {
                                println!("Failed to send transaction to channel");
                                break;
                            }
                        } else if let Ok(block_id) = serde_json::from_str::<BlockId>(&msg) {
                            // if the message is a BlockId, send it to the block_request channel
                            if block_request_tx_clone.send(block_id).is_err() {
                                println!("Failed to send block request to channel");
                                break;
                            }
                        } else {
                            println!("Received unknown message: {}", msg);
                        }
                    }
                    Err(e) => {
                        println!("Failed to read message from {}: {}", peer_addr, e);
                        break;
                    }
                }
            }
    
            println!("Closing connection to {}", peer_addr);
        });
    }

    /// Creates a new P2PNetwork instance and associated FIFO communication channels.
    /// There are 5 FIFO channels. 
    /// Those channels are used for communication within the process.
    /// They abstract away the network and neighbor nodes. 
    /// More specifically, they are for communicating between `bin_nakamoto` threads 
    /// and threads that are responsible for TCP network communication.
    /// The usage of those five channels can be guessed from the type:
    /// 1. Receiver<BlockNode>: read from this FIFO channel to receive blocks from the network.
    /// 2. Receiver<Transaction>: read from this FIFO channel to receive transactions from the network.
    /// 3. Sender<BlockNode>: write to this FIFO channel to broadcast a block to the network.
    /// 4. Sender<Transaction>: write to this FIFO channel to broadcast a transaction to the network.
    /// 5. Sender<BlockId>: write to this FIFO channel to request a block from the network.
    pub fn create(address: NetAddress, neighbors: Vec<NetAddress>) -> (
        Arc<Mutex<P2PNetwork>>,
        Receiver<BlockNode>, 
        Receiver<Transaction>, 
        Sender<BlockNode>, 
        Sender<Transaction>,
        Sender<BlockId>
    ) {
        // Please fill in the blank
        // You might need to perform the following steps:
        // 1. create a P2PNetwork instance
        // 2. create mpsc channels for sending and receiving messages
        // 3. create a thread for accepting incoming TCP connections from neighbors
        // 4. create TCP connections to all neighbors
        // 5. create threads for each TCP connection to send messages
        // 6. create threads to listen to messages from neighbors
        // 7. create threads to distribute received messages (send to channels or broadcast to neighbors)
        // 8. return the created P2PNetwork instance and the mpsc channels
        //todo!();

        // 1. create a P2PNetwork instance
        let p2p_network = P2PNetwork {
            send_msg_count: 0,
            recv_msg_count: 0,
            address,
            neighbors,
        };
        
        // 2. create mpsc channels for sending and receiving messages
        let (block_node_tx, block_node_rx) = channel();
        let (transaction_tx, transaction_rx) = channel();
        let (block_broadcast_tx, block_broadcast_rx) = channel();
        let (transaction_broadcast_tx, transaction_broadcast_rx) = channel();
        let (block_request_tx, block_request_rx) = channel();
        
        let p2p_network = Arc::new(Mutex::new(p2p_network));
        let p2p_network_clone = p2p_network.clone();
       
       //Step 3 to 7

       // 3. create a thread to accept incoming TCP connections from neighbors
       
        
        // 8. return the created P2PNetwork instance and the mpsc channels
        (
            p2p_network,
            block_node_rx,
            transaction_rx,
            block_broadcast_tx,
            transaction_broadcast_tx,
            block_request_tx
        )

        
    }

    /// Get status information of the P2PNetwork for debug printing.
    pub fn get_status(&self) -> BTreeMap<String, String> {
        // Please fill in the blank
        // For debugging purpose, you can return any dictionary of strings as the status of the network. 
        // It should be displayed in the Client UI eventually.
        todo!();
        
    }

}


