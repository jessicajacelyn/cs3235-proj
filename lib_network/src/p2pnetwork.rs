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
use std::io::{BufReader, Read, Result, BufWriter, Write, BufRead};
use std::{convert, io};
use std::net::{TcpListener, TcpStream, SocketAddr, ToSocketAddrs, SocketAddrV4, Ipv4Addr};
use std::sync::mpsc::{Receiver, Sender, channel, TryRecvError};
use std::thread;
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;
use futures::select;

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
            address: address.clone(),
            neighbors: neighbors.clone(),
        };
        
        // 2. create mpsc channels for sending and receiving messages
        // let (block_node_tx, block_node_rx) = channel();
        // let (transaction_tx, transaction_rx) = channel();
        // let (block_broadcast_tx, block_broadcast_rx) = channel();
        // let (transaction_broadcast_tx, transaction_broadcast_rx) = channel();
        // let (block_request_tx, block_request_rx) = channel();
        
        // let p2p_network = Arc::new(Mutex::new(p2p_network));
        // let p2p_network_clone = p2p_network.clone();      

        let (block_sender, block_receiver) = channel();
        let (tx_sender, tx_receiver) = channel();
        let (block_id_sender, block_id_receiver) = channel();

        // 3. create a thread for accepting incoming TCP connections from neighbors
        
        // let socket_string = format!("{}:{}", &address.ip, &address.port);
        // let tcp_listener = TcpListener::bind(socket_string).expect("failed to bind TCP listener");

        let p2p_network = Arc::new(Mutex::new(p2p_network));
        let p2p_clone = p2p_network.clone();
        let block_sender_clone: Sender<BlockNode> = block_sender.clone();
        let tx_sender_clone: Sender<Transaction> = tx_sender.clone();
        let block_id_sender_clone : Sender<BlockId> = block_id_sender.clone();
        thread::spawn(move || {
            let socket_string = format!("{}:{}", &address.ip, &address.port);
            let listener = TcpListener::bind(socket_string).expect("failed to bind TCP listener");
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let p2p = p2p_clone.clone();
                        let block_sender = block_sender_clone.clone();
                        let tx_sender = tx_sender_clone.clone();
                        let block_id_sender = block_id_sender_clone.clone();
                        thread::spawn(move || {
                            let mut reader = BufReader::new(&stream);
                            let mut writer = BufWriter::new(&stream);

                            loop {
                                let mut msg = String::new();
                                match reader.read_line(&mut msg) {
                                    Ok(_) => {
                                        let parts: Vec<&str> = msg.trim().split(":").collect();
                                        match parts[0] {
                                            "block" => {
                                                let block = serde_json::from_str(parts[1]).unwrap();
                                                block_sender.send(block).unwrap();
                                            }
                                            "tx" => {
                                                let tx = serde_json::from_str(parts[1]).unwrap();
                                                tx_sender.send(tx).unwrap();
                                            }
                                            "block_id" => {
                                                let block_id = serde_json::from_str(parts[1]).unwrap();
                                                block_id_sender.send(block_id).unwrap();
                                            }
                                            _ => {}
                                        }
                                    }
                                    Err(_) => {
                                        break;
                                    }
                                }
                            }
                        });
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
            }
        });

        
        // 4. create TCP connections to all neighbors
        let mut senders: Vec<Sender<String>> = Vec::new();

        for neighbor in &neighbors {
            let socket_string = format!("{}:{}", &neighbor.ip, &neighbor.port);
            match TcpStream::connect(socket_string) {
                Ok(stream) => {
                    let (sender, receiver) = channel();
                    senders.push(sender);
                    // Spawn a thread to send messages over the channel
                    std::thread::spawn(move || {
                        let mut writer = BufWriter::new(&stream);
                        loop {
                            match receiver.recv() {
                                Ok(msg) => {
                                    if let Err(e) = writer.write(msg.as_bytes()) {
                                        println!("Error: {}", e);
                                        break;
                                    }
                                    if let Err(e) = writer.flush() {
                                        println!("Error: {}", e);
                                        break;
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                    });
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
        
        // 5. create threads for each TCP connection to send messages

        
        todo!()
        
    }

    /// Get status information of the P2PNetwork for debug printing.
    pub fn get_status(&self) -> BTreeMap<String, String> {
        // Please fill in the blank
        // For debugging purpose, you can return any dictionary of strings as the status of the network. 
        // It should be displayed in the Client UI eventually.
        todo!();
        
    }

}


