extern crate pnet;

use pnet::datalink::Channel::Ethernet;
use pnet::datalink::{self, DataLinkReceiver, DataLinkSender, MacAddr, NetworkInterface};
use pnet::packet::ethernet::{EtherType, EthernetPacket, MutableEthernetPacket};
use pnet::packet::{MutablePacket, Packet};
use std::collections::HashMap;
use std::sync::mpsc;
use std::{env, thread};

enum Payload {
    Packet {
        source: MacAddr,
        destination: MacAddr,
        kind: EtherType,
    },
    Pulse(u64),
}

fn watch_interface(
    mut tx: Box<DataLinkSender>,
    mut rx: Box<DataLinkReceiver>,
    snd: mpsc::SyncSender<Payload>,
) {
    loop {
        match rx.next() {
            Ok(packet) => {
                let packet = EthernetPacket::new(packet).unwrap();

                {
                    let payload: Payload = Payload::Packet {
                        source: packet.get_source(),
                        destination: packet.get_destination(),
                        kind: packet.get_ethertype(),
                    };
                    let thr_snd = snd.clone();
                    thread::spawn(move || {
                        thr_snd.send(payload).unwrap();
                    });
                }

                tx.build_and_send(1, packet.packet().len(), &mut |new_packet| {
                    let mut new_packet = MutableEthernetPacket::new(new_packet).unwrap();

                    // Create a clone of the original packet
                    new_packet.clone_from(&packet);

                    // Switch the source and destination
                    new_packet.set_source(packet.get_destination());
                    new_packet.set_destination(packet.get_source());
                });
            }
            Err(e) => {
                panic!("An error occurred while reading: {}", e);
            }
        }
    }
}

fn timer(snd: mpsc::SyncSender<Payload>) -> () {
    use std::{thread, time};
    let one_second = time::Duration::from_millis(1000);

    let mut pulses = 0;
    loop {
        thread::sleep(one_second);
        snd.send(Payload::Pulse(pulses)).unwrap();
        pulses += 1;
    }
}

fn gather(rcv: mpsc::Receiver<Payload>) -> () {
    let mut sources: HashMap<MacAddr, u64> = HashMap::new();
    let mut destinations: HashMap<MacAddr, u64> = HashMap::new();
    let mut ethertypes: HashMap<EtherType, u64> = HashMap::new();

    while let Ok(payload) = rcv.recv() {
        match payload {
            Payload::Pulse(id) => {
                println!("REPORT {}", id);
                println!("    SOURCES:");
                for (k, v) in sources.iter() {
                    println!("        {}: {}", k, v);
                }
                println!("    DESTINATIONS:");
                for (k, v) in destinations.iter() {
                    println!("        {}: {}", k, v);
                }
                println!("    ETHERTYPES:");
                for (k, v) in ethertypes.iter() {
                    println!("        {}: {}", k, v);
                }
            }
            Payload::Packet {
                source: src,
                destination: dst,
                kind: etype,
            } => {
                let mut destination = destinations.entry(dst).or_insert(0);
                *destination += 1;

                let mut source = sources.entry(src).or_insert(0);
                *source += 1;

                let mut ethertype = ethertypes.entry(etype).or_insert(0);
                *ethertype += 1;
            }
        }
    }
}

fn main() {
    let interface_name = env::args().nth(1).unwrap();
    let interface_names_match = |iface: &NetworkInterface| iface.name == interface_name;

    // Find the network interface with the provided name
    let interfaces: Vec<NetworkInterface> = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .filter(interface_names_match)
        .next()
        .unwrap();

    let (snd, rcv) = mpsc::sync_channel(10);

    let _ = thread::spawn(|| gather(rcv));
    let timer_snd = snd.clone();
    let _ = thread::spawn(move || timer(timer_snd));

    let iface_handler = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => {
            let snd = snd.clone();
            thread::spawn(|| watch_interface(tx, rx, snd))
        }
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!(
            "An error occurred when creating the datalink channel: {}",
            e
        ),
    };

    iface_handler.join().unwrap();
}
