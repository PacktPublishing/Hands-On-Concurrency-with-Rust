extern crate pnet;
extern crate rlua;

use pnet::datalink::Channel::Ethernet;
use pnet::datalink::{self, DataLinkReceiver, MacAddr, NetworkInterface};
use pnet::packet::ethernet::{EtherType, EthernetPacket};
use rlua::{prelude, Function, Lua};
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::{env, thread};

static SKIPPED_PACKETS: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
enum Payload {
    Packet {
        source: MacAddr,
        destination: MacAddr,
        kind: EtherType,
    },
    Pulse(u64),
}

fn watch_interface(mut rx: Box<DataLinkReceiver>, snd: mpsc::SyncSender<Payload>) {
    loop {
        match rx.next() {
            Ok(packet) => {
                let packet = EthernetPacket::new(packet).unwrap();

                let payload: Payload = Payload::Packet {
                    source: packet.get_source(),
                    destination: packet.get_destination(),
                    kind: packet.get_ethertype(),
                };
                if snd.try_send(payload).is_err() {
                    SKIPPED_PACKETS.fetch_add(1, Ordering::Relaxed);
                }
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

fn eval<'a>(lua: &'a Lua, path: &Path, name: Option<&str>) -> prelude::LuaResult<Function<'a>> {
    let f = File::open(path).unwrap();
    let mut reader = BufReader::new(f);

    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).unwrap();
    let s = ::std::str::from_utf8(&buf).unwrap();

    lua.eval(s, name)
}

fn main() {
    let interface_name = env::args().nth(1).unwrap();
    let pulse_fn_file_arg = env::args().nth(2).unwrap();
    let pulse_fn_file = Path::new(&pulse_fn_file_arg);
    assert!(pulse_fn_file.exists());
    let interface_names_match = |iface: &NetworkInterface| iface.name == interface_name;

    // Find the network interface with the provided name
    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .filter(interface_names_match)
        .next()
        .unwrap();

    let lua = Lua::new();
    let pulse_fn = eval(&lua, pulse_fn_file, Some("pulse")).unwrap();

    let (snd, rcv) = mpsc::sync_channel(100);

    let timer_snd = snd.clone();
    let _ = thread::spawn(move || timer(timer_snd));

    let _iface_handler = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(_tx, rx)) => {
            let snd = snd.clone();
            thread::spawn(|| watch_interface(rx, snd))
        }
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!(
            "An error occurred when creating the datalink channel: {}",
            e
        ),
    };

    let destinations = lua.create_table().unwrap();
    let sources = lua.create_table().unwrap();
    let kinds = lua.create_table().unwrap();

    while let Ok(payload) = rcv.recv() {
        match payload {
            Payload::Packet {
                source,
                destination,
                kind,
            } => {
                let d_cnt = destinations.get(destination.to_string()).unwrap_or(0);
                destinations
                    .set(destination.to_string(), d_cnt + 1)
                    .unwrap();

                let s_cnt = sources.get(source.to_string()).unwrap_or(0);
                sources.set(source.to_string(), s_cnt + 1).unwrap();

                let k_cnt = kinds.get(kind.to_string()).unwrap_or(0);
                kinds.set(kind.to_string(), k_cnt + 1).unwrap();
            }
            Payload::Pulse(id) => {
                let skipped_packets = SKIPPED_PACKETS.swap(0, Ordering::Relaxed);
                pulse_fn
                    .call::<_, ()>((
                        id,
                        skipped_packets,
                        destinations.clone(),
                        sources.clone(),
                        kinds.clone(),
                    ))
                    .unwrap()
            }
        }
    }
}
