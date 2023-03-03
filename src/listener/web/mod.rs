//! WebSocket connection support.

use axum::extract::ws::{Message, WebSocket};
use axum::extract::WebSocketUpgrade;
use axum::response::IntoResponse;
use axum::Extension;
use axum_client_ip::InsecureClientIp;
use crossbeam::channel::Sender;
use futures_util::StreamExt;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::thread::JoinHandle;
use tokio::runtime::Runtime;

use super::ServerMessage;

/// Start a thread that will accept UDP packets and message them
/// to the server's event loop.
pub fn start_web_thread(addr: &str, sender: Sender<ServerMessage>) -> JoinHandle<()> {
    println!("[web] Starting WebSocket server at {}", addr);

    let addr: SocketAddr = addr.parse().unwrap();

    std::thread::spawn(move || {
        let runtime = Runtime::new().unwrap();
        runtime.block_on(server(addr, sender));
    })
}

async fn server(addr: SocketAddr, sender: Sender<ServerMessage>) {
    use axum::{routing::get, Router};

    let state = Arc::new(State { sender });

    let router = Router::new()
        .route("/", get(get_websocket))
        .layer(Extension(state));

    axum::Server::bind(&addr).serve(router.into_make_service_with_connect_info::<SocketAddr>());
}

struct State {
    sender: Sender<ServerMessage>,
}

async fn get_websocket(
    ws: WebSocketUpgrade,
    Extension(state): Extension<Arc<State>>,
    InsecureClientIp(addr): InsecureClientIp,
) -> impl IntoResponse {
    let sender = state.sender.clone();
    ws.on_upgrade(move |ws| websocket(ws, sender, addr))
}

/// Single WebSocket connection.
async fn websocket(ws: WebSocket, sender: Sender<ServerMessage>, ip: IpAddr) {
    println!("[web] WebSocket opening.");
    let (_tx, mut rx) = ws.split();

    while let Some(result) = rx.next().await {
        match result {
            Ok(msg) => {
                match msg {
                    Message::Binary(data) => {
                        match sender.send(ServerMessage::Binary { ip, data }) {
                            Ok(_) => {}
                            Err(_) => {
                                // listener dead?
                                break;
                            }
                        }
                    }
                    _ => {
                        eprintln!("Non-binary message?");
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
