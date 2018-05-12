#[macro_use]
extern crate slog;
extern crate clap;
extern crate slog_async;
extern crate slog_term;
extern crate threadpool;

use clap::{App, Arg};
use slog::Drain;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicUsize, Ordering};
use threadpool::ThreadPool;

static TOTAL_STREAMS: AtomicUsize = AtomicUsize::new(0);

fn handle_client(
    log: slog::Logger,
    mut reader: BufReader<TcpStream>,
    mut writer: BufWriter<TcpStream>,
) -> () {
    let mut buf = String::with_capacity(2048);

    while let Ok(sz) = reader.read_line(&mut buf) {
        info!(log, "Received a {} bytes: {}", sz, buf);
        writer
            .write_all(&buf.as_bytes())
            .expect("could not write line");
        buf.clear();
    }
    TOTAL_STREAMS.fetch_sub(1, Ordering::Relaxed);
}

fn main() {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let root = slog::Logger::root(drain, o!());

    let matches = App::new("server")
        .arg(
            Arg::with_name("host")
                .long("host")
                .value_name("HOST")
                .help("Sets which hostname to listen on")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .long("port")
                .value_name("PORT")
                .help("Sets which port to listen on")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("max_connections")
                .long("max_connections")
                .value_name("CONNECTIONS")
                .help("Sets how many connections (thus, threads) to allow simultaneously")
                .takes_value(true),
        )
        .get_matches();

    let host: &str = matches.value_of("host").unwrap_or("localhost");
    let port = matches
        .value_of("port")
        .unwrap_or("1987")
        .parse::<u16>()
        .expect("port-no not valid");
    let max_connections = matches
        .value_of("max_connections")
        .unwrap_or("256")
        .parse::<u16>()
        .expect("max_connections not valid");

    let listener = TcpListener::bind((host, port)).unwrap();
    let server = root.new(o!("host" => host.to_string(), "port" => port));
    info!(server, "Server open for business! :D");

    let pool: ThreadPool = threadpool::Builder::new()
        .num_threads(max_connections as usize)
        .build();

    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            if pool.active_count() == (max_connections as usize) {
                info!(
                    server,
                    "Max connection condition reached, rejecting incoming"
                );
            } else {
                let stream_no = TOTAL_STREAMS.fetch_add(1, Ordering::Relaxed);
                let log = root.new(o!("stream-no" => stream_no,
                   "peer-addr" => stream.peer_addr().expect("no peer address").to_string()));
                let writer = BufWriter::new(stream.try_clone().expect("could not clone stream"));
                let reader = BufReader::new(stream);
                pool.execute(move || handle_client(log, reader, writer));
            }
        } else {
            info!(root, "Shutting down! {:?}", stream);
        }
    }

    info!(
        server,
        "No more incoming connections. Draining existing connections."
    );
    pool.join();
}
