use event;
use std::{net, thread};
use std::net::ToSocketAddrs;
use std::str;
use std::str::FromStr;
use std::sync::mpsc;
use util;

pub struct IngestPoint {
    host: String,
    port: u16,
    chans: Vec<mpsc::Sender<event::Event>>,
}

impl IngestPoint {
    pub fn init(
        host: String,
        port: u16,
        chans: Vec<mpsc::Sender<event::Event>>,
    ) -> IngestPoint {
        IngestPoint {
            chans: chans,
            host: host,
            port: port,
        }
    }

    pub fn run(&mut self) {
        let mut joins = Vec::new();

        let addrs = (self.host.as_str(), self.port).to_socket_addrs();
        if let Ok(ips) = addrs {
            let ips: Vec<_> = ips.collect();
            for addr in ips {
                let listener =
                    net::UdpSocket::bind(addr).expect("Unable to bind to UDP socket");
                let chans = self.chans.clone();
                joins.push(thread::spawn(move || handle_udp(chans, &listener)));
            }
        }

        for jh in joins {
            jh.join().expect("Uh oh, child thread panicked!");
        }
    }
}

fn parse_packet(buf: &str) -> Option<event::Telemetry> {
    let mut iter = buf.split_whitespace();
    if let Some(name) = iter.next() {
        if let Some(val) = iter.next() {
            match u32::from_str(val) {
                Ok(int) => {
                    return Some(event::Telemetry {
                        name: name.to_string(),
                        value: int,
                    })
                }
                Err(_) => return None,
            };
        }
    }
    None
}

fn handle_udp(mut chans: Vec<mpsc::Sender<event::Event>>, socket: &net::UdpSocket) {
    let mut buf = vec![0; 16_250];
    loop {
        let (len, _) = match socket.recv_from(&mut buf) {
            Ok(r) => r,
            Err(e) => panic!(format!("Could not read UDP socket with error {:?}", e)),
        };
        if let Some(telem) = parse_packet(str::from_utf8(&buf[..len]).unwrap()) {
            util::send(&mut chans, event::Event::Telemetry(telem));
        }
    }
}
