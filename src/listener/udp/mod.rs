//! UDP server implementation.

use crossbeam::channel::Sender;
use std::net::UdpSocket;
use std::thread::{self, JoinHandle};

const MAX_PACKET_SIZE: usize = 4096;

use super::ServerMessage;

/// Start a thread that will accept UDP packets and message them
/// to the server's event loop.
pub fn start_udp_thread(udp_addr: &str, sender: Sender<ServerMessage>) -> JoinHandle<()> {
    println!("[udp] Starting UDP server at {}", udp_addr);
    let socket = UdpSocket::bind(udp_addr).expect("[udp] Unable to create UDP socket!");

    thread::spawn(move || loop {
        let mut buf = [0; MAX_PACKET_SIZE];
        let (len, source) = socket.recv_from(&mut buf).unwrap();
        // println!("[udp] recv {} B", len);

        let slice = &(buf)[0..len];

        let message = ServerMessage::Binary {
            ip: source.ip(),
            data: slice.to_owned(),
        };

        sender
            .send(message)
            .expect("[udp] Packet receiver gone. Exiting thread.");
    })
}
