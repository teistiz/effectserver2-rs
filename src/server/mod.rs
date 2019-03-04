use std::io;
use std::net::{IpAddr, UdpSocket};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{self, JoinHandle};

use crate::config::Root;
use crate::mapper::Mapper;

const MAX_PACKET_SIZE: usize = 4096;

enum ServerMessage {
    Binary { ip: IpAddr, data: Vec<u8> },
}

/// Takes a configuration and a command Mapper and
pub fn serve(config: Root, mut mapper: Mapper) -> io::Result<()> {
    let (sender, receiver) = channel::<ServerMessage>();

    let udp_handle = start_udp_thread(
        &config.server.udp_addr,
        sender.clone(),
    );

    'message_loop: loop {
        match receiver.recv() {
            Ok(request) => match request {
                ServerMessage::Binary { ip, data } => {
                    match mapper.take_msg(data.as_slice()) {
                        Ok(_) => {
                            // ...
                        },
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

    udp_handle.join()
        .expect("Did the UDP thread crash?");

    Ok(())
}

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
