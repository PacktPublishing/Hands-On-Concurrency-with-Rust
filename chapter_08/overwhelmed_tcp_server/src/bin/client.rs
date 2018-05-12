#[macro_use]
extern crate slog;
extern crate clap;
extern crate slog_async;
extern crate slog_term;

use clap::{App, Arg};
use slog::Drain;
use std::net::TcpStream;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{thread, time};

static TOTAL_STREAMS: AtomicUsize = AtomicUsize::new(0);

fn report(log: slog::Logger) {
    let delay = time::Duration::from_millis(1000);
    let mut total_streams = 0;
    loop {
        let streams_per_second = TOTAL_STREAMS.swap(0, Ordering::Relaxed);
        info!(log, "Total connections: {}", total_streams);
        info!(log, "Connections per second: {}", streams_per_second);
        total_streams += streams_per_second;
        thread::sleep(delay);
    }
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
        .get_matches();

    let host: &str = matches.value_of("host").unwrap_or("localhost");
    let port = matches
        .value_of("port")
        .unwrap_or("1987")
        .parse::<u16>()
        .expect("port-no not valid");

    let client = root.new(o!("host" => host.to_string(), "port" => port));
    info!(client, "Client ready to be mean. >:)");

    let reporter_log = root.new(o!("meta" => "report"));
    let _ = thread::spawn(|| report(reporter_log));

    let mut streams = Vec::with_capacity(2048);
    loop {
        match TcpStream::connect((host, port)) {
            Ok(stream) => {
                TOTAL_STREAMS.fetch_add(1, Ordering::Relaxed);
                streams.push(stream);
            }
            Err(err) => error!(client, "Connection rejected with error: {:?}", err),
        }
    }
}
