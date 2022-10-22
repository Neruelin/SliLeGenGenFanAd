// use tonic_ws_transport::WsConnection;
// use tower::ServiceBuilder;
// // use std::time::Duration;
// use tokio::net::{TcpListener, TcpStream};

// use futures_util::{StreamExt, TryStreamExt, TryFutureExt};
// use tokio;
// use tokio_stream::wrappers::TcpListenerStream;
// use tonic::{transport::Server, Request, Response, Status};

use goblin::goblin_game_server::{GoblinGame, GoblinGameServer};
use goblin::{GetPlayerLocationRequest, GetPlayerLocationResponse};

use game::game_game_server::{GameGame, GameGameServer};
use game::{ClSendChatMessage, ClSetNameResult};
use tokio::io::Interest;


pub mod game {
    tonic::include_proto!("game");
}

// #[derive(Debug, Default)]
// pub struct GameGameService {}

// #[tonic::async_trait]

// impl GameGame for GameGameService {
//     async fn test_it(
//         &self,
//         request: Request<ClSendChatMessage>
//     ) -> Result<Response<ClSetNameResult>, Status> {
//         println!("got chat message");
//         let reply = ClSetNameResult {
//             success: true
//         };
//         Ok(Response::new(reply))
//     }
// }

pub mod goblin {
    tonic::include_proto!("goblin");
}

// #[derive(Debug, Default)]
// pub struct GoblinGameService {}

// #[tonic::async_trait]
// impl GoblinGame for GoblinGameService {
//     async fn get_player_location(
//         &self, 
//         request: Request<GetPlayerLocationRequest>
//     ) -> Result<Response<GetPlayerLocationResponse>, Status> {
//         println!("got player location request!");
//         let reply = GetPlayerLocationResponse {
//             successful: true,
//             x: 5,
//             y: 10
//         };
//         Ok(Response::new(reply))
//     }
// }

// use tokio::net::TcpListener;
// use tonic::transport::server::Connected;

use std::io;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
};
use sha1::{Sha1, Digest};
use base64::encode;
use httparse;

fn make_ws_accept(s: String) -> String {
    let mag = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let concat = s + mag;
    let mut hasher = Sha1::new();
    hasher.update(concat.as_bytes());
    encode(hasher.finalize())
}

fn handle_request(lines: String) -> String {
    println!("request:\n{:?}", lines);
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut req = httparse::Request::new(&mut headers);
    let res = req.parse(lines.as_bytes()).unwrap();
    if res.is_complete() {
        let mut wskey = String::new();
        for h in req.headers {
            match h.name {
                "Sec-WebSocket-Key" => {
                    wskey = String::from_utf8(h.value.to_vec()).unwrap();
                },
                _ => {}
            }
        }
        let ws_accept = make_ws_accept(wskey);

        let mut response = String::from("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: ");
        response += &ws_accept;
        response += "\r\n\r\n";
        println!("response:\n{:?}", response);
        response
    } else {
        "".to_owned()
    }
}

fn print_bytes(buf: & Vec<u8>) {
    let mut s = String::new();
    for i in buf {
        for j in 0..8 {
            if i & 1<<j > 0 {
                s += "1";
            } else {
                s += "0";
            }
        }
        s += "\n";
    }
    println!("{}", s);
}

#[tokio::main]
async fn main() -> io::Result<()> {

    let listener = TcpListener::bind("127.0.0.1:4000").await?;

    loop {
        let (mut _socket, _addr) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            let (reader, mut writer) = _socket.split();
            let mut raw_line = vec![];
            let mut lines = String::new();

            loop {
                reader.ready(Interest::READABLE).await;
                match reader.try_read_buf(&mut raw_line) {
                    Ok(bytes_read) => {
                        if bytes_read == 0 {
                            break;
                        }
                        let line = std::str::from_utf8(&raw_line).unwrap();
                        lines += line;
                        if line == "\r\n" {
                            raw_line.clear();
                            break;
                        }
                        raw_line.clear();
                    },
                    Err(e) => {
                        println!("{:?}", e);
                        break;
                    }
                }
            }
            let resp = handle_request(lines);
            writer.write_all(resp.as_bytes()).await.unwrap();

            // writer.write_all(&[0x9; 1]).await.unwrap();

            loop {
                println!("waiting for data");
                if let Ok(r) = reader.ready(Interest::READABLE).await {
                    println!("reading data");
                    match reader.try_read_buf(&mut raw_line) {
                        Ok(n) => {
                            if n == 0 {
                                break;
                            }
                            println!("{:x?}", raw_line);
                            print_bytes(&raw_line);
                            // let line = std::str::from_utf8(&raw_line).unwrap();
                            // println!("{:?} : {:?}", n, line);
                        },
                        Err(e) => {
                            println!("err {:?}", e);
                            break;
                        }
                    }
                }
            }
        });
    }

    Ok(())
}