//! Monitoring view for the effect mapper.

use axum::{
    extract::ws::WebSocket,
    http::{header, StatusCode},
    response::{Html, IntoResponse},
    Extension,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Serialize, Serializer};
use std::{thread::JoinHandle, net::IpAddr};
use std::{net::SocketAddr, sync::Arc};
use tokio::runtime::Runtime;
use tokio::sync::watch::Receiver;

const HTML_DOC: &str = include_str!("../../../static/index.html");
const HTML_SCRIPT: &str = include_str!("../../../static/script.js");

#[derive(Serialize, Debug, Clone)]
pub struct StatusMessage {
    pub lights: Vec<LightStatus>,
}

#[derive(Serialize, Debug, Clone)]
pub struct LightStatus {
    pub id: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    #[serde(serialize_with="serialize_tag")]
    pub tag: Option<Arc<String>>,
    #[serde(serialize_with="serialize_ip")]
    pub ip: Option<IpAddr>,
}

pub fn serialize_tag<S: Serializer>(tag: &Option<Arc<String>>, s: S) -> Result<S::Ok, S::Error> {
    match tag {
        Some(arc_str) => {
            s.collect_str(arc_str.as_str())
        },
        None => {
            s.serialize_none()
        }
    }
}
pub fn serialize_ip<S: Serializer>(tag: &Option<IpAddr>, s: S) -> Result<S::Ok, S::Error> {
    match tag {
        Some(addr) => {
            s.collect_str(&addr.to_string())
        },
        None => {
            s.serialize_none()
        }
    }
}
pub fn start_monitor_thread(addr: &str, receiver: Receiver<StatusMessage>) -> JoinHandle<()> {
    println!("[monitor] Starting monitoring server at {}", addr);
    let addr: SocketAddr = addr.parse().unwrap();

    std::thread::spawn(move || {
        let runtime = Runtime::new().unwrap();
        runtime.block_on(server(addr, receiver));
    })
}

/// Shared state that can be injected into method handlers.
struct ServerState {
    receiver: Receiver<StatusMessage>,
}

async fn server(addr: SocketAddr, receiver: Receiver<StatusMessage>) {
    use axum::{routing::get, Router};

    let state = Arc::new(ServerState { receiver });

    let app = Router::new()
        .route("/", get(get_page))
        .route("/ws", get(get_websocket))
        .route("/script.js", get(get_script))
        .layer(Extension(state));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap()
}

async fn get_page() -> impl IntoResponse {
    Html(HTML_DOC)
}

async fn get_script() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/javascript")],
        HTML_SCRIPT,
    )
}

async fn get_websocket(
    ws: axum::extract::WebSocketUpgrade,
    Extension(state): Extension<Arc<ServerState>>,
) -> impl IntoResponse {
    let recv = state.receiver.clone();
    ws.on_upgrade(move |ws| handle_websocket(ws, recv))
}

async fn handle_websocket(ws: WebSocket, mut recv: Receiver<StatusMessage>) {
    use axum::extract::ws::Message;

    let (mut tx, _rx) = ws.split();

    loop {
        // watch-type queues have an initial value.
        let message = Message::Text(serde_json::to_string(&*recv.borrow()).unwrap());
        match tx.send(message).await {
            Ok(_) => {}
            Err(_err) => {
                eprintln!("[monitor] WebSocket dropped?");
                return;
            }
        }
        if !recv.changed().await.is_ok() {
            break;
        }
    }
    println!("[monitor] WebSocket terminating.");
}
