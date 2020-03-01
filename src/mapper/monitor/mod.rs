//! Monitoring view for the effect mapper.

use futures::sink::SinkExt;
use futures::stream::StreamExt;
use serde::Serialize;
use std::net::SocketAddr;
use std::thread::JoinHandle;
use tokio::runtime::Runtime;
use tokio::sync::watch::Receiver;
use warp::{
    http::Response,
    ws::{Message, WebSocket},
    Filter,
};

const HTML_DOC: &str = include_str!("./monitor.html");
const HTML_SCRIPT: &str = include_str!("./monitor.js");

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
}

pub fn start_monitor_thread(addr: &str, receiver: Receiver<StatusMessage>) -> JoinHandle<()> {
    println!("[monitor] Starting monitoring server at {}", addr);
    let addr: SocketAddr = addr.parse().unwrap();

    std::thread::spawn(move || {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(server(addr, receiver));
    })
}

async fn server(addr: SocketAddr, receiver: Receiver<StatusMessage>) {
    let root = warp::get()
        .and(warp::path::end())
        .map(|| warp::reply::html(HTML_DOC));

    use warp::path;

    let r_script = warp::get().and(path!("monitor.js")).map(|| {
        Response::builder()
            .header("Content-Type", "text/javascript")
            .body(HTML_SCRIPT)
            .unwrap()
    });

    let ws = warp::ws().map(move |ws: warp::filters::ws::Ws| {
        let receiver = receiver.clone();
        ws.on_upgrade(move |ws| websocket(ws, receiver))
    });

    warp::serve(ws.or(r_script).or(root)).run(addr).await;
}

async fn websocket(ws: WebSocket, mut recv: Receiver<StatusMessage>) {
    println!("[monitor] WebSocket opening.");

    let (mut tx, _rx) = ws.split();

    while let Some(status) = recv.recv().await {
        match tx
            .send(Message::text(serde_json::to_string(&status).unwrap()))
            .await
        {
            Ok(_) => {}
            Err(_err) => {
                eprintln!("[monitor] WebSocket dropped?");
                return;
            }
        }
    }
    println!("[monitor] WebSocket terminating.");
}
