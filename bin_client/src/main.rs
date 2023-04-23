// This file is part of the project for the module CS3235 by Prateek
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

/// This is the client program that covers the following tasks:
/// 1. File I/O. Read the config file and state files for initialization, dump the state files, etc.
/// 2. Read user input (using terminal UI) about transaction creation or quitting.
/// 3. Display the status and logs to the user (using terminal UI).
/// 4. IPC communication with the bin_nakamoto and the bin_wallet processes.
use seccompiler;
use seccompiler::{BpfMap, BpfProgram};

use tui::{backend::CrosstermBackend, Terminal};
use tui_textarea::{Input, Key};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::process::{Command, Stdio};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use serde_json;
use std::sync::{Arc, Mutex};
use std::{
    thread,
    time::{Duration, Instant},
};

use std::fs;

mod app;

/// The enum type for the IPC messages (requests) from this client to the bin_nakamoto process.
/// It is the same as the `IPCMessageRequest` enum type in the bin_nakamoto process.
#[derive(Serialize, Deserialize, Debug, Clone)]
enum IPCMessageReqNakamoto {
    Initialize(String, String, String),
    GetAddressBalance(String),
    PublishTx(String, String),
    RequestBlock(String),
    RequestNetStatus,
    RequestChainStatus,
    RequestMinerStatus,
    RequestTxPoolStatus,
    RequestStateSerialization,
    Quit,
}

/// The enum type for the IPC messages (responses) from the bin_nakamoto process to this client.
/// It is the same as the enum type in the bin_nakamoto process.
#[derive(Serialize, Deserialize, Debug, Clone)]
enum IPCMessageRespNakamoto {
    Initialized,
    PublishTxDone,
    AddressBalance(String, i64),
    BlockData(String),
    NetStatus(BTreeMap<String, String>),
    ChainStatus(BTreeMap<String, String>),
    MinerStatus(BTreeMap<String, String>),
    TxPoolStatus(BTreeMap<String, String>),
    StateSerialization(String, String),
    Quitting,
    Notify(String),
}

/// The enum type for the IPC messages (requests) from this client to the bin_wallet process.
/// It is the same as the enum type in the bin_wallet process.
#[derive(Serialize, Deserialize, Debug, Clone)]
enum IPCMessageReqWallet {
    Initialize(String),
    Quit,
    SignRequest(String),
    VerifyRequest(String, String),
    GetUserInfo,
}

/// The enum type for the IPC messages (responses) from the bin_wallet process to this client.
/// It is the same as the enum type in the bin_wallet process.
#[derive(Serialize, Deserialize, Debug, Clone)]
enum IPCMessageRespWallet {
    Initialized,
    Quitting,
    SignResponse(String, String),
    VerifyResponse(bool, String),
    UserInfo(String, String),
}

/// The enum type representing bot commands for controlling the client automatically.
/// The commands are read from a file or a named pipe and then executed by the client.
#[derive(Serialize, Deserialize, Debug, Clone)]
enum BotCommand {
    /// Send a transaction message from the default user_id of the client to the given receiver_user_id, e.g, Send(`receiver_user_id`, `transaction_message`)
    Send(String, String),
    /// Wait for the given number of milliseconds, e.g., SleepMs(`milliseconds`)
    SleepMs(u64),
}

/// Read a file and return the content as a string.
fn read_string_from_file(filepath: &str) -> String {
    let contents = fs::read_to_string(filepath).expect(&("Cannot read ".to_owned() + filepath));
    contents
}

/// A flag indicating whether to disable the UI thread if you need to check some debugging outputs that is covered by the UI.
/// Eventually this should be set to false and you shouldn't output debugging information directly to stdout or stderr.
const NO_UI_DEBUG_NODE: bool = false;

fn main() {
    // The usage of bin_client is as follows:
    // bin_client <client_seccomp_path> <nakamoto_config_path> <nakamoto_seccomp_path> <wallet_config_path> <wallet_seccomp_path> [<bot_command_path>]
    // - `client_seccomp_path`: The path to the seccomp file for this client process for Part B. (You can set this argument to any value during Part A.)
    // - `nakamoto_config_path`: The path to the config folder for the bin_nakamoto process. For example, `./tests/nakamoto_config1`. Your program should read the 3 files in the config folder (`BlockTree.json`, `Config.json`, `TxPool.json`) for initializing bin_nakamoto.
    // - `nakamoto_seccomp_path`: The path to the seccomp file for the bin_nakamoto process for Part B. (You can set this argument to any value during Part A.)
    // - `wallet_config_path`: The path to the config file for the bin_wallet process. For example, `./tests/_secrets/Walley.A.json`. Your program should read the file for initializing bin_wallet.
    // - `wallet_seccomp_path`: The path to the seccomp file for the bin_wallet process for Part B. (You can set this argument to any value during Part A.)
    // - [`bot_command_path`]: *Optional* argument. The path to the file or named pipe for the bot commands. If this argument is provided, your program should read commands line-by-line from the file.
    //                         an example file of the bot commands can be found at `./tests/_bots/botA-0.jsonl`. You can also look at `run_four.sh` for an example of using the named pipe version of this argument.
    //                         The bot commands are executed by the client in the order they are read from the file or the named pipe.
    //                         The bot commands should be executed in a separate thread so that the UI thread can still be responsive.
    // Please fill in the blank
    // - Create bin_nakamoto process:  Command::new("./target/debug/bin_nakamoto")...
    // - Create bin_wallet process:  Command::new("./target/debug/bin_wallet")...
    // - Get stdin and stdout of those processes
    // - Create buffer readers if necessary
    // - Send initialization requests to bin_nakamoto and bin_wallet

    // Create bin_nakamoto process
    let mut bin_nakamoto = Command::new("./target/debug/bin_nakamoto")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn bin_nakamoto process");

    // Create bin_wallet process
    let mut bin_wallet = Command::new("./target/debug/bin_wallet")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn bin_wallet process");

    // Get stdin and stdout of bin_nakamoto process
    let nakamoto_stdin_p = Arc::new(Mutex::new(
        bin_nakamoto
            .stdin
            .take()
            .expect("Failed to get stdin of bin_nakamoto"),
    ));
    let nakamoto_stdout = bin_nakamoto
        .stdout
        .take()
        .expect("Failed to get stdout of bin_nakamoto");

    let nakamoto_stderr = bin_nakamoto
        .stderr
        .take()
        .expect("Failed to get stderr of bin_nakamoto");

    // Get stdin and stdout of bin_wallet process
    let bin_wallet_stdin_p = Arc::new(Mutex::new(
        bin_wallet
            .stdin
            .take()
            .expect("Failed to get stdin of bin_wallet"),
    ));
    let bin_wallet_stdout = bin_wallet
        .stdout
        .take()
        .expect("Failed to get stdout of bin_wallet");

    let bin_wallet_stderr = bin_wallet
        .stderr
        .take()
        .expect("Failed to get stderr of bin_wallet");

    // Create buffer readers if necessary
    let mut bin_nakamoto_reader = std::io::BufReader::new(nakamoto_stdout);
    let mut bin_wallet_reader = std::io::BufReader::new(bin_wallet_stdout);
    let mut nakamoto_stderr_reader = std::io::BufReader::new(nakamoto_stderr);
    let mut wallet_stderr_reader = std::io::BufReader::new(bin_wallet_stderr);

    // Read folder path and get the files from the folder
    let folder_path = std::env::args().nth(2).unwrap();
    let files = fs::read_dir(folder_path).unwrap();
    let first_file = files
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
        .unwrap()[0]
        .to_str()
        .unwrap()
        .to_string();
    let folder_path = std::env::args().nth(2).unwrap();
    let files = fs::read_dir(folder_path).unwrap();
    let second_file = files
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
        .unwrap()[1]
        .to_str()
        .unwrap()
        .to_string();
    let folder_path = std::env::args().nth(2).unwrap();
    let files = fs::read_dir(folder_path).unwrap();
    let third_file = files
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
        .unwrap()[2]
        .to_str()
        .unwrap()
        .to_string();

    // Send initialization requests to bin_wallet
    let wallet_init_request = IPCMessageReqWallet::Initialize(read_string_from_file(
        std::env::args().nth(4).unwrap().as_str(),
    ));
    let wallet_init_request_str = serde_json::to_string(&wallet_init_request).unwrap();
    writeln!(
        bin_wallet_stdin_p.lock().unwrap(),
        "{}",
        wallet_init_request_str
    )
    .expect("Failed to write to bin_wallet stdin");

    // Send initialization requests to bin_nakamoto
    let nakamoto_init_request = IPCMessageReqNakamoto::Initialize(
        read_string_from_file(&first_file),
        read_string_from_file(&second_file),
        read_string_from_file(&third_file),
    );
    let nakamoto_init_request_str = serde_json::to_string(&nakamoto_init_request).unwrap();
    writeln!(
        nakamoto_stdin_p.lock().unwrap(),
        "{}",
        nakamoto_init_request_str
    )
    .expect("Failed to write to bin_nakamoto stdin");

    let client_seccomp_path = std::env::args()
        .nth(1)
        .expect("Please specify client seccomp path");

    // Please fill in the blank
    // sandboxing the bin_client (For part B). Leave it blank for part A.

    let user_name: String;
    let user_id: String;
    // Please fill in the blank
    // Read the user info from wallet
    let get_user_info_request = IPCMessageReqWallet::GetUserInfo;
    let get_user_info_request_str = serde_json::to_string(&get_user_info_request).unwrap();

    writeln!(
        bin_wallet_stdin_p.lock().unwrap(),
        "{}",
        get_user_info_request_str
    )
    .expect("Failed to write to bin_wallet stdin");

    let mut wallet_response = String::new();
    bin_wallet_reader
        .read_line(&mut wallet_response)
        .expect("Failed to read from bin_wallet stdout");
    let wallet_response: IPCMessageRespWallet = serde_json::from_str(&wallet_response).unwrap();
    match wallet_response {
        IPCMessageRespWallet::UserInfo(name, id) => {
            user_name = name;
            user_id = id;
        }
        _ => panic!("Unexpected response from wallet"),
    }

    // Create the Terminal UI app
    let app_arc = Arc::new(Mutex::new(app::App::new(
        user_name.clone(),
        user_id.clone(),
        "".to_string(),
        format!("SEND $100   // By {}", user_name),
    )));

    // An enclosure func to generate signing requests when creating new transactions.
    let create_sign_req = |sender: String, receiver: String, message: String| {
        let timestamped_message = format!(
            "{}   // {}",
            message,
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
        let sign_req = IPCMessageReqWallet::SignRequest(
            serde_json::to_string(&(sender, receiver, timestamped_message)).unwrap(),
        );
        let mut sign_req_str = serde_json::to_string(&sign_req).unwrap();
        sign_req_str.push('\n');
        return sign_req_str;
    };

    // This is optional so .... nvm ....
    if std::env::args().len() != 6 {
        // Then there must be 7 arguments provided. The last argument is the bot commands path
        // Please fill in the blank
        // Create a thread to read the bot commands from `bot_command_path`, execute those commands and update the UI
        // Notice that the `SleepMs(1000)` doesn't mean that the all threads in the whole process should sleep for 1000ms. It means that
        // The next bot command that fakes the user interaction should be processed 1000ms later.
        // It should not block the execution of any other threads or the main thread.
        let bot_command_path = std::env::args().nth(6).unwrap();

        // Spawn a separate thread to read and execute bot commands
        // thread::spawn(move || {
        // Open the bot command file
        // let file = File::open(bot_command_path).expect("Failed to open bot command file");
        // let reader = BufReader::new(file);

        // // Read bot commands line by line
        // for line in reader.lines() {
        //     if let Ok(command_str) = line {
        //         // Parse the command string into a BotCommand struct
        //         let bot_command = match parse_bot_command(&command_str) {
        //             Some(cmd) => cmd,
        //             None => {
        //                 println!("Failed to parse bot command: {}", command_str);
        //                 continue;
        //             }
        //         };

        //         // Execute the bot command and update the app state
        //         {
        //             // Lock the app state with the mutex
        //             let mut app = app_arc.lock().expect("Failed to acquire app mutex");

        //             // Match on the bot command and execute it
        //             match bot_command {
        //                 BotCommand::Send(receiver_user_id, transaction_message) => {
        //                     // Execute the Send command and update the app state
        //                     /* execute Send command and update app state */
        //                 }
        //                 BotCommand::SleepMs(milliseconds) => {
        //                     // Sleep for the specified number of milliseconds
        //                     thread::sleep(Duration::from_millis(milliseconds));
        //                 }
        //             }

        //             // Release the mutex to allow other threads to acquire it
        //         }
        //     }
        // }
        // });
    }

    // Please fill in the blank
    // - Spawn threads to read/write from/to bin_nakamoto/bin_wallet. (Through their piped stdin and stdout)
    // - You should request for status update from bin_nakamoto periodically (every 500ms at least) to update the App (UI struct) accordingly.
    // - You can also create threads to read from stderr of bin_nakamoto/bin_wallet and add those lines to the UI (app.stderr_log) for easier debugging.

    // Spawn a thread to read SignResponse from bin_wallet and send it to bin_nakamoto
    {
        let nakamoto_stdin_p = nakamoto_stdin_p.clone();
        thread::spawn(move || {
            loop {
                let mut wallet_response = String::new();
                bin_wallet_reader
                    .read_line(&mut wallet_response)
                    .expect("Failed to read from bin_wallet stdout");
                let wallet_response: IPCMessageRespWallet =
                    serde_json::from_str(&wallet_response).unwrap();
                match wallet_response {
                    IPCMessageRespWallet::SignResponse(data_string, signature) => {
                        // send to bin_nakamoto
                        let mut nakamoto_stdin = nakamoto_stdin_p.lock().unwrap();
                        nakamoto_stdin
                            .write_all(
                                format!(
                                    "{}\n",
                                    serde_json::to_string(&IPCMessageReqNakamoto::PublishTx(
                                        data_string,
                                        signature
                                    ))
                                    .unwrap()
                                )
                                .as_bytes(),
                            )
                            .expect("Failed to write to bin_nakamoto stdin");
                    }
                    _ => panic!("Unexpected response from wallet"),
                }
            }
        });
    }

    // Spawn a thread to read from stderr of bin_nakamoto and bin_wallet and add those lines to the UI (app.stderr_log) for easier debugging.
    {
        let app_arc = app_arc.clone();
        thread::spawn(move || loop {
            let mut nakamoto_stderr = String::new();
            nakamoto_stderr_reader
                .read_line(&mut nakamoto_stderr)
                .expect("Failed to read from bin_nakamoto stderr");
            let mut app = app_arc.lock().expect("Failed to acquire app mutex");
            app.stderr_log.push(nakamoto_stderr);

            let mut wallet_stderr = String::new();
            wallet_stderr_reader
                .read_line(&mut wallet_stderr)
                .expect("Failed to read from bin_wallet stderr");
            app.stderr_log.push(wallet_stderr);
        });
    }

    // Spawn a thread to periodically request for status update from bin_nakamoto
    {
        let nakamoto_stdin_p = nakamoto_stdin_p.clone();
        let app_arc = app_arc.clone();
        thread::spawn(move || {
            loop {
                // Get AddressBalance from bin_nakamoto
                let address_balance_request =
                    IPCMessageReqNakamoto::GetAddressBalance(user_id.clone());
                let address_balance_request_str =
                    serde_json::to_string(&address_balance_request).unwrap();
                let mut nakamoto_stdin = nakamoto_stdin_p.lock().unwrap();
                nakamoto_stdin
                    .write_all(format!("{}\n", address_balance_request_str).as_bytes())
                    .expect("Failed to write to bin_nakamoto stdin");

                // Update UI from response
                let mut app = app_arc.lock().expect("Failed to acquire app mutex");
                let mut nakamoto_stdout = String::new();
                bin_nakamoto_reader
                    .read_line(&mut nakamoto_stdout)
                    .expect("Failed to read from bin_nakamoto stdout");
                let nakamoto_stdout: IPCMessageRespNakamoto =
                    serde_json::from_str(&nakamoto_stdout).unwrap();
                match nakamoto_stdout {
                    IPCMessageRespNakamoto::AddressBalance(_user_id, address_balance) => {
                        app.user_balance = address_balance;
                    }
                    _ => panic!("Unexpected response from nakamoto"),
                }

                // Get status from bin_nakamoto
                let chain_status_request = IPCMessageReqNakamoto::RequestChainStatus;
                let chain_status_request_str =
                    serde_json::to_string(&chain_status_request).unwrap();
                writeln!(
                    nakamoto_stdin_p.lock().unwrap(),
                    "{}",
                    chain_status_request_str
                )
                .expect("Failed to write to bin_nakamoto stdin");

                let net_status_request = IPCMessageReqNakamoto::RequestNetStatus;
                let net_status_request_str = serde_json::to_string(&net_status_request).unwrap();
                writeln!(
                    nakamoto_stdin_p.lock().unwrap(),
                    "{}",
                    net_status_request_str
                )
                .expect("Failed to write to bin_nakamoto stdin");

                let miner_status_request = IPCMessageReqNakamoto::RequestMinerStatus;
                let miner_status_request_str =
                    serde_json::to_string(&miner_status_request).unwrap();
                writeln!(
                    nakamoto_stdin_p.lock().unwrap(),
                    "{}",
                    miner_status_request_str
                )
                .expect("Failed to write to bin_nakamoto stdin");

                let pool_status_request = IPCMessageReqNakamoto::RequestTxPoolStatus;
                let pool_status_request_str = serde_json::to_string(&pool_status_request).unwrap();
                writeln!(
                    nakamoto_stdin_p.lock().unwrap(),
                    "{}",
                    pool_status_request_str
                )
                .expect("Failed to write to bin_nakamoto stdin");

                let mut nakamoto_response = String::new();
                bin_nakamoto_reader
                    .read_line(&mut nakamoto_response)
                    .expect("Failed to read from bin_nakamoto stdout");
                let nakamoto_response: IPCMessageRespNakamoto =
                    serde_json::from_str(&nakamoto_response).unwrap();

                match nakamoto_response {
                    IPCMessageRespNakamoto::ChainStatus(status) => {
                        app_arc.lock().unwrap().blocktree_status = status;
                    }
                    IPCMessageRespNakamoto::NetStatus(status) => {
                        app_arc.lock().unwrap().network_status = status;
                    }
                    IPCMessageRespNakamoto::MinerStatus(status) => {
                        app_arc.lock().unwrap().miner_status = status;
                    }
                    IPCMessageRespNakamoto::TxPoolStatus(status) => {
                        app_arc.lock().unwrap().txpool_status = status;
                    }
                    _ => panic!("Unexpected response from nakamoto"),
                };

                // Sleep for 500ms
                thread::sleep(Duration::from_millis(500));
            }
        });
    }

    // UI thread. Modify it to suit your needs.
    let app_ui_ref = app_arc.clone();
    let bin_wallet_stdin_p_cloned = bin_wallet_stdin_p.clone();
    let nakamoto_stdin_p_cloned = nakamoto_stdin_p.clone();
    let handle_ui = thread::spawn(move || {
        let tick_rate = Duration::from_millis(200);
        if NO_UI_DEBUG_NODE {
            // If app_ui.should_quit is set to true, the UI thread will exit.
            loop {
                if app_ui_ref.lock().unwrap().should_quit {
                    break;
                }
                // sleep for 500ms
                thread::sleep(Duration::from_millis(500));
            }
            return;
        }
        let ui_loop = || -> Result<(), io::Error> {
            // setup terminal
            enable_raw_mode()?;
            let mut stdout = io::stdout();
            execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
            let backend = CrosstermBackend::new(stdout);
            let mut terminal = Terminal::new(backend)?;

            let mut last_tick = Instant::now();
            loop {
                terminal.draw(|f| app_ui_ref.lock().unwrap().draw(f))?;

                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_millis(100));

                if crossterm::event::poll(timeout)? {
                    let input = event::read()?.into();
                    let mut app = app_ui_ref.lock().unwrap();
                    match input {
                        Input { key: Key::Esc, .. } => {
                            app.on_quit();
                        }
                        Input { key: Key::Down, .. } => app.on_down(),
                        Input { key: Key::Up, .. } => app.on_up(),
                        Input {
                            key: Key::Enter, ..
                        } => {
                            if !app.are_inputs_valid {
                                app.client_log("Invalid inputs! Cannot create Tx.".to_string());
                            } else {
                                let (sender, receiver, message) = app.on_enter();
                                let sign_req_str = create_sign_req(sender, receiver, message);
                                bin_wallet_stdin_p_cloned
                                    .lock()
                                    .unwrap()
                                    .write_all(sign_req_str.as_bytes())
                                    .unwrap();
                            }
                        }
                        // on control + s, request Nakamoto to serialize its state
                        Input {
                            key: Key::Char('s'),
                            ctrl: true,
                            ..
                        } => {
                            let serialize_req = IPCMessageReqNakamoto::RequestStateSerialization;
                            let nakamoto_stdin = nakamoto_stdin_p_cloned.clone();
                            let mut to_send = serde_json::to_string(&serialize_req).unwrap();
                            to_send.push_str("\n");
                            nakamoto_stdin
                                .lock()
                                .unwrap()
                                .write_all(to_send.as_bytes())
                                .unwrap();
                        }
                        input => {
                            app.on_textarea_input(input);
                        }
                    }
                }

                let mut app = app_ui_ref.lock().unwrap();
                if last_tick.elapsed() >= tick_rate {
                    app.on_tick();
                    last_tick = Instant::now();
                }
                if app.should_quit {
                    break;
                }
            }
            // restore terminal
            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
            terminal.show_cursor()?;
            Ok(())
        };
        ui_loop().unwrap();
    });
    handle_ui.join().unwrap();

    eprintln!("--- Sending \"Quit\" command...");
    nakamoto_stdin_p
        .lock()
        .unwrap()
        .write_all("\"Quit\"\n".as_bytes())
        .unwrap();
    bin_wallet_stdin_p
        .lock()
        .unwrap()
        .write_all("\"Quit\"\n".as_bytes())
        .unwrap();

    // Please fill in the blank
    // Wait for IPC threads to finish

    let ecode1 = bin_nakamoto
        .wait()
        .expect("failed to wait on child nakamoto");
    eprintln!("--- nakamoto ecode: {}", ecode1);

    let ecode2 = bin_wallet
        .wait()
        .expect("failed to wait on child bin_wallet");
    eprintln!("--- bin_wallet ecode: {}", ecode2);
}
