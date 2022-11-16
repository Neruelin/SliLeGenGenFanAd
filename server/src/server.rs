use futures::SinkExt;
use futures_util::{StreamExt};
use log::*;
use ws_messages::ws_messages::{CreateControlEntityMessage, DisconnectMessage};
use std::{net::SocketAddr, time::Duration};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Error};
use tungstenite::{Message, Result};
use serde_json;
use std::fs::File;
use std::sync::{mpsc, Arc, RwLock};

mod ws_messages;
use crate::ws_messages::ws_messages::{RawMessage, MessageType, GetEntitiesRequestMessage, NewEntityRequestMessage, ClearEntitiesRequestMessage, MoveEntityRequestMessage};
mod game_state;
use crate::game_state::game_state::{game_loop, Obj, Interaction};
use std::convert::TryFrom;

fn handle_message(msg: Vec<u8>, tx: mpsc::Sender<Interaction>, objss: &Arc<RwLock<Vec<Obj>>>) -> Option<Message> {
    let rm = RawMessage { data: msg };
    let rm_type = rm.get_msg_type().unwrap();
    match rm_type {
        MessageType::GetEntitiesRequest => {
            let ent_req = GetEntitiesRequestMessage::try_from(rm).unwrap();
            {
                let objsss = objss.read().unwrap();
                Some(GetEntitiesRequestMessage::create_response(objsss, ent_req.x, ent_req.y, ent_req.w, ent_req.h))
            }
        },
        MessageType::NewEntityRequest => {
            let new_ent_req = NewEntityRequestMessage::try_from(rm).unwrap();
            tx.send(Interaction::CreateEntity { x: new_ent_req.x, y: new_ent_req.y }).unwrap();
            None
        },
        MessageType::ClearEntitiesRequest => {
            let new_clear_ent_req = ClearEntitiesRequestMessage::try_from(rm).unwrap();
            tx.send(Interaction::ClearEntities {}).unwrap();
            None
        },
        MessageType::MoveEntityRequest => {
            let new_move_ent_req = MoveEntityRequestMessage::try_from(rm).unwrap();
            tx.send(Interaction::MoveEntity { x: new_move_ent_req.x, y: new_move_ent_req.y, id: new_move_ent_req.entity_id });
            None
        }
        MessageType::CreateControlEntity => {
            let new_create_control_ent_req = CreateControlEntityMessage::try_from(rm).unwrap();
            let (_tx, rx) = mpsc::channel();
            tx.send(Interaction::CreateControlEntity { tx: _tx });
            let mut owned_obj = rx.recv().unwrap();
            Some(CreateControlEntityMessage::create_response(owned_obj))
        },
        MessageType::Disconnect => {
            let new_disconnect_req = DisconnectMessage::try_from(rm).unwrap();
            tx.send(Interaction::DisconnectControlEntity { id: new_disconnect_req.id });
            None
        }
        _ => {None}
    }
}

async fn accept_connection(peer: SocketAddr, stream: TcpStream, tx: mpsc::Sender<Interaction>, objss: Arc<RwLock<Vec<Obj>>>) {
    if let Err(e) = handle_connection(peer, stream, tx, objss).await {
        match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
            err => error!("Error processing connection: {}", err),
        }
    }
}

async fn handle_connection(peer: SocketAddr, stream: TcpStream, tx: mpsc::Sender<Interaction>, objss: Arc<RwLock<Vec<Obj>>>) -> Result<()> {
    let ws_stream = accept_async(stream).await.expect("Failed to accept");
    println!("New WebSocket connection: {}", peer);
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let mut interval = tokio::time::interval(Duration::from_millis(1000));

    loop {
        tokio::select! {
            msg = ws_receiver.next() => {
                match msg {
                    Some(msg) => {
                        match msg {
                            Ok(msg) => {
                                match msg {
                                    Message::Binary(m) => {
                                        let retval = handle_message(m, tx.clone(), &objss);
                                        match retval {
                                            Some(vv) => {
                                                ws_sender.send(vv).await?;
                                            },
                                            None => {}
                                        }
                                    },
                                    Message::Close(close_frame) => {
                                        println!("Peer disconnected");
                                    },
                                    _ => { 
                                        println!("dunno what this is? {:?}", msg);
                                    }
                                }
                            },
                            _ => {}
                        }
                    }
                    None => break,
                }
            }
            _ = interval.tick() => {
                // let mut data: [u8; 3] = [i as u8 % 2, commands[i % commands.len()], commands[j % commands.len()]];
                // ws_sender.send(Message::Binary(data.to_vec())).await?;
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

    let (tx, rx) = mpsc::channel();

    let objss: Arc<RwLock<Vec<Obj>>> = Arc::new(RwLock::new(vec![]));
    let objssloopclone = Arc::clone(&objss);
    std::thread::spawn(move || {
        game_loop(rx, objssloopclone);
    });

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream.peer_addr().expect("connected streams should have a peer address");
        println!("Peer address: {}", peer);

        tokio::spawn(accept_connection(peer, stream, tx.clone(), Arc::clone(&objss)));
    }
}