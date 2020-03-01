//! Accepts UDP and other things from the network.

use std::io;
use std::net::{IpAddr};
use crossbeam::channel;

use crate::config::Root;
use crate::mapper::Mapper;

mod udp;
mod web;

/// Message formats that can be received by the server(s).
#[derive(Debug)]
pub enum ServerMessage {
    Binary { ip: IpAddr, data: Vec<u8> },
}

/// Start an API for a pre-configured Mapper.
pub fn serve(config: Root, mut mapper: Mapper) -> io::Result<()> {
    // Message channel used as the server's event bus.
    let (sender, receiver) = channel::unbounded::<ServerMessage>();

    // Start the server(s).
    let udp_handle = udp::start_udp_thread(&config.server.udp_addr, sender.clone());
    let web_handle = web::start_web_thread(&config.server.websocket_addr, sender.clone());

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

    web_handle.join().expect("Did the UDP thread crash?");
    udp_handle.join().expect("Did the UDP thread crash?");

    Ok(())
}
