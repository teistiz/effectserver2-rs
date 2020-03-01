//! WebSocket connection support.

use crossbeam::channel::Sender;
use futures::stream::StreamExt;
use std::net::SocketAddr;
use std::thread::JoinHandle;
use tokio::runtime::Runtime;
use warp::filters::ws::WebSocket;
use warp::Filter;

use super::ServerMessage;

/// Start a thread that will accept UDP packets and message them
/// to the server's event loop.
pub fn start_web_thread(addr: &str, sender: Sender<ServerMessage>) -> JoinHandle<()> {
    println!("[web] Starting Web server at {}", addr);

    let addr: SocketAddr = addr.parse().unwrap();

    std::thread::spawn(move || {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(server(addr, sender));
    })
}

async fn server(addr: SocketAddr, sender: Sender<ServerMessage>) {
    let with_sender = warp::any().map(move || sender.clone());
    let with_ip = warp::addr::remote().map(|addr: Option<SocketAddr>| {
        addr
    });

    let ws = warp::ws().and(with_sender).and(with_ip).map(
        |ws: warp::filters::ws::Ws, sender: Sender<ServerMessage>, addr: Option<SocketAddr>| {
            let sender = sender.clone();
            let addr = addr.clone();
            ws.on_upgrade(move |ws| websocket(ws, sender, addr.unwrap()))
        },
    );

    warp::serve(ws).run(addr).await;
}

/// Single WebSocket connection.
async fn websocket(ws: WebSocket, sender: Sender<ServerMessage>, addr: SocketAddr) {
    println!("[web] WebSocket opening.");
    // let ip = ws.
    let (_tx, mut rx) = ws.split();

    while let Some(result) = rx.next().await {
        match result {
            Ok(msg) => {
                if msg.is_binary() {
                    match sender.send(ServerMessage::Binary {
                        ip: addr.ip(),
                        data: msg.into_bytes(),
                    }) {
                        Ok(_) => {}
                        Err(_) => {
                            // listener dead?
                            break;
                        }
                    }
                }
            }
            Err(_) => {
                eprintln!("[web] WebSocket error?");
                break;
            }
        }
    }

    println!("[web] WebSocket terminating.");
}
