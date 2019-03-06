//! Accepts UDP and other things from the network.

use std::io;
use std::net::{IpAddr, UdpSocket};
use std::sync::mpsc::{channel, Sender};
use std::thread::{self, JoinHandle};

use crate::config::Root;
use crate::mapper::Mapper;

const MAX_PACKET_SIZE: usize = 4096;

/// Message formats that can be received by the server(s).
enum ServerMessage {
    Binary { ip: IpAddr, data: Vec<u8> },
}

/// Start an API for a pre-configured Mapper.
pub fn serve(config: Root, mut mapper: Mapper) -> io::Result<()> {
    // Message channel used as the server's event bus.
    let (sender, receiver) = channel::<ServerMessage>();

    // Start the server(s).
    let udp_handle = start_udp_thread(&config.server.udp_addr, sender.clone());

    // Listen to messages from the server(s) and pass them to the mapper.
    'message_loop: loop {
        match receiver.recv() {
            Ok(request) => match request {
                ServerMessage::Binary { ip, data } => {
                    match mapper.take_msg(data.as_slice(), Some(ip)) {
                        Ok(_) => {
                            // ...
                        }
                        Err(err) => {
                            eprintln!("msg fail: {:?}", err);
                        }
                    }
                }
            },
            Err(err) => {
                eprintln!("{:?}", err);
                break 'message_loop;
            }
        }
    }

    udp_handle.join().expect("Did the UDP thread crash?");

    Ok(())
}

/// Start a thread that will accept UDP packets and message them
/// to the server's event loop.
fn start_udp_thread(udp_addr: &str, sender: Sender<ServerMessage>) -> JoinHandle<()> {
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
