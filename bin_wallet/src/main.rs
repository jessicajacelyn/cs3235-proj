// This file is part of the project for the module CS3235 by Prateek
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

// This is the main file for the bin_wallet binary
// It is a simple command-line program that can be used to sign and verify messages
// It reads from stdin and writes to stdout to facilitate IPC communication with bin_client eventually.
// However, you can run it directly from the command line to test it.
// You can see detailed instructions in the comments below.

mod wallet;
use seccompiler::*;
use serde::{Deserialize, Serialize};
use serde_json::*;
use std::fs;
use std::io::{self, BufRead, Write};

/// Read a string from a file (help with debugging)
fn read_string_from_file(filepath: &str) -> String {
    let contents = fs::read_to_string(filepath).expect(&("Cannot read ".to_owned() + filepath));
    contents
}

/// Write a string to a file (to help you debug)
fn write_string_to_file(filepath: &str, content: String) {
    fs::write(filepath, content).expect(&("Cannot write ".to_owned() + filepath));
}

/// Append a string to a file (to help you debug)
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

/// The enum representing IPC message requests from the stdin
#[derive(Serialize, Deserialize, Debug, Clone)]
enum IPCMessageReq {
    /// Quit the execution
    Quit,
    /// Initialize the wallet by deserializing the provided json string
    Initialize(String),
    /// Sign the provided data string using the private key
    SignRequest(String),
    /// Verify the provided (`data_string`, `signature_in_base64`) using the public key
    VerifyRequest(String, String),
    /// Get the user info
    GetUserInfo,
}

/// The enum representing IPC message responses to the stdout
#[derive(Serialize, Deserialize, Debug, Clone)]
enum IPCMessageResp {
    /// The wallet has been initialized
    Initialized,
    /// The wallet is quitting normally
    Quitting,
    /// The response to a sign request (DataString, Signature)
    SignResponse(String, String),
    /// The response to a verify request (isSuccess, DataString)
    VerifyResponse(bool, String),
    /// The response to the get user info request (username, user_id). User Id is transformed from the public key.
    UserInfo(String, String),
}

fn main() {
    // bin_wallet has only one optional argument: the path to the seccomp policy file
    // If the argument is provided, bin_wallet will read and apply the seccomp policy at the beginning of the program
    // Otherwise, it will proceed to the normal execution
    let maybe_policy_path = std::env::args().nth(1);
    if let Some(policy_path) = maybe_policy_path {
        // If the first param is provided, read the seccomp config and apply it
        let filter_map: BpfMap = seccompiler::compile_from_json(
            read_string_from_file(&policy_path).as_bytes(),
            std::env::consts::ARCH.try_into().unwrap(),
        )
        .unwrap();
        let filter = filter_map.get("main_thread").unwrap();

        seccompiler::apply_filter(&filter).unwrap();
    }

    // The main logic of the bin_wallet starts here
    // It reads IPC calls from stdin and write IPC responses to stdout in a loop.
    // The first IPC call is always the Initialize call with the wallet data provided.
    // After that, there can be arbitrary number of SignRequest, VerifyRequest, and GetUserInfo calls.
    // Eventually, the Quit call will be received and the program will exit.
    use wallet::Wallet;
    // Please fill in the blank
    //todo!();
    let mut wallet: Option<Wallet> = None;
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let input = line.unwrap();
        let request: IPCMessageReq =
            serde_json::from_str(&input).expect("Failed to parse input as IPCMessageReq");
        let response = match request {
            IPCMessageReq::Quit => IPCMessageResp::Quitting,
            IPCMessageReq::Initialize(wallet_json) => {
                wallet = Some(
                    serde_json::from_str(&wallet_json)
                        .expect("Failed to parse wallet_json as Wallet"),
                );
                IPCMessageResp::Initialized
            }
            IPCMessageReq::SignRequest(data) => {
                let wallet = wallet.as_ref().expect("Wallet not initialized");
                let signature = wallet.sign(&data);
                IPCMessageResp::SignResponse(data, signature)
            }
            IPCMessageReq::VerifyRequest(data, signature) => {
                let wallet = wallet.as_ref().expect("Wallet not initialized");
                let is_valid = wallet.verify(&data, &signature);
                IPCMessageResp::VerifyResponse(is_valid, data)
            }
            IPCMessageReq::GetUserInfo => {
                let wallet = wallet.as_ref().expect("Wallet not initialized");
                let user_id = wallet.get_user_id();
                let username = wallet.get_user_name();
                IPCMessageResp::UserInfo(username, user_id)
            }
        };
        let output = serde_json::to_string(&response).unwrap();
        println!("{}\n", output);
        // if request == IPCMessageReq::Quit {
        //     println!("{}\n", serde_json::to_string(&IPCMessageResp::Quitting).unwrap());
        //     break;
        // }
    }

    //println!("{}\n", serde_json::to_string(&IPCMessageResp::Quitting).unwrap());
}

#[cfg(test)]
mod test {
    use crate::{
        read_string_from_file, wallet::Wallet, write_string_to_file, IPCMessageReq, IPCMessageResp,
    };

    /// This test generates a new wallet and writes it to a file.
    #[test]
    fn generate_new_wallet() {
        let bin_wallet = Wallet::new("Haha".to_string(), 384);
        let bin_wallet_str = serde_json::to_string_pretty(&bin_wallet).unwrap();
        write_string_to_file("../tests/_secrets/Wallet.new.json", bin_wallet_str);
    }

    /// This test reads a wallet from a file and uses it to sign and verify a message.
    #[test]
    fn test_bin_wallet_signing_and_verifying() {
        let bin_wallet: Wallet =
            serde_json::from_str(&read_string_from_file("../tests/_secrets/Wallet.B.json"))
                .unwrap();
        println!("Private key Pem:\n{}\n", bin_wallet.priv_key_pem);
        println!("Public key Pem:\n{}\n", bin_wallet.pub_key_pem);
        let msg = "hello world";
        let sig64 = bin_wallet.sign(msg);

        let verify_result = bin_wallet.verify(msg, &sig64);
        println!("msg: {}\nsig64: {}\nverify: {}", msg, sig64, verify_result);
    }

    /// This test reads a wallet from a file and uses it to verify a message signed by a reference implementation.
    #[test]
    fn test_bin_wallet_verifying_alice() {
        let msg = "[\"MDgCMQCqrJ1yIJ7cDQIdTuS+4CkKn/tQPN7bZFbbGCBhvjQxs71f6Vu+sD9eh8JGpfiZSckCAwEAAQ==\",\"MDgCMQDOpK8YWmcg8ffNF/O7xlBDq/DBdoUnc4yyWrV0y/X3LF+dddjaGksXzGl3tHskpgkCAwEAAQ==\",\"SEND $300   // By Alice   // 1678250102871\"]".to_string();
        let sig = "l8gsKxmAUzhgqbVqGlXaO69+Qhr87QthvZjUbYZXvnb+tanxCi8wm3c5UjHZ+HKm".to_string();
        let bin_wallet: Wallet =
            serde_json::from_str(&read_string_from_file("../tests/_secrets/Wallet.A.json"))
                .unwrap();
        let verify_result = bin_wallet.verify(&msg, &sig);
        println!("msg: {}\nsig64: {}\nverify: {}", msg, sig, verify_result);
        assert!(verify_result);

        let sig2 = "58gsKxmAUzhgqbVqGlXaO69+Qhr87QthvZjUbYZXvnb+tanxCi8wm3c5UjHZ+HKm".to_string();
        let verify_result = bin_wallet.verify(&msg, &sig2);
        assert!(!verify_result);
    }
}
