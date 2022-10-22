use futures_util::{SinkExt, StreamExt};
use log::*;
use tower::load;
use std::{net::SocketAddr, time::Duration};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Error};
use tungstenite::{Message, Result};
use serde_json;
use std::fs::File;

async fn accept_connection(peer: SocketAddr, stream: TcpStream) {
    if let Err(e) = handle_connection(peer, stream).await {
        match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
            err => error!("Error processing connection: {}", err),
        }
    }
}

async fn handle_connection(peer: SocketAddr, stream: TcpStream) -> Result<()> {
    let ws_stream = accept_async(stream).await.expect("Failed to accept");
    println!("New WebSocket connection: {}", peer);
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let mut interval = tokio::time::interval(Duration::from_millis(1000));

    // Echo incoming WebSocket messages and send a message periodically every second.

    let mut i = 0;
    let mut j = 2;

    let commands = [0, 1, 2, 2, 2, 1, 0, 0]; // values are +1 because we can only send u8

    loop {
        tokio::select! {
            msg = ws_receiver.next() => {
                match msg {
                    Some(msg) => {
                        let msg = msg?;
                        if msg.is_text() ||msg.is_binary() {
                            ws_sender.send(msg).await?;
                        } else if msg.is_close() {
                            break;
                        }
                    }
                    None => break,
                }
            }
            _ = interval.tick() => {
                println!("sent");
                let data: [u8; 3] = [1, commands[i % commands.len()], commands[j % commands.len()]];
                ws_sender.send(Message::Binary(data.to_vec())).await?;
                i += 1;
                j += 1;
            }
        }
    }

    Ok(())
}

const CONFIG_FILE_PATH: &str = "../config.json";

fn load_config_addr() -> String {
    let config_file = File::open(CONFIG_FILE_PATH).unwrap();
    let json_config: serde_json::Value = serde_json::from_reader(config_file).unwrap();
    let mut addr: String = json_config["server_addr"].as_str().unwrap().to_string();
    addr += ":";
    addr += json_config["server_port"].as_str().unwrap();
    addr
}

#[tokio::main]
async fn main() {
    let addr = load_config_addr();    
    println!("Listening on: {}", addr);
    let listener = TcpListener::bind(&addr).await.expect("Can't listen");

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream.peer_addr().expect("connected streams should have a peer address");
        println!("Peer address: {}", peer);

        tokio::spawn(accept_connection(peer, stream));
    }
}