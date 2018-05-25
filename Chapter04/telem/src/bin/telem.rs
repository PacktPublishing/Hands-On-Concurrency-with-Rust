extern crate telem;

use std::{thread, time};
use std::sync::mpsc;
use telem::IngestPoint;
use telem::egress::{CKMSEgress, CMAEgress, Egress};
use telem::event::Event;
use telem::filter::{Filter, HighFilter, LowFilter};

fn main() {
    let limit = 100;
    let (lp_ic_snd, lp_ic_rcv) = mpsc::channel::<Event>();
    let (hp_ic_snd, hp_ic_rcv) = mpsc::channel::<Event>();
    let (ckms_snd, ckms_rcv) = mpsc::channel::<Event>();
    let (cma_snd, cma_rcv) = mpsc::channel::<Event>();

    let filter_sends = vec![lp_ic_snd, hp_ic_snd];
    let ingest_filter_sends = filter_sends.clone();
    let _ingest_jh = thread::spawn(move || {
        IngestPoint::init("127.0.0.1".to_string(), 1990, ingest_filter_sends).run();
    });
    let _low_jh = thread::spawn(move || {
        let mut low_filter = LowFilter::new(limit);
        low_filter.run(lp_ic_rcv, vec![ckms_snd]);
    });
    let _high_jh = thread::spawn(move || {
        let mut high_filter = HighFilter::new(limit);
        high_filter.run(hp_ic_rcv, vec![cma_snd]);
    });
    let _ckms_egress_jh = thread::spawn(move || {
        CKMSEgress::new(0.01).run(ckms_rcv);
    });
    let _cma_egress_jh = thread::spawn(move || {
        CMAEgress::new().run(cma_rcv);
    });

    let one_second = time::Duration::from_millis(1_000);
    loop {
        for snd in &filter_sends {
            snd.send(Event::Flush).unwrap();
        }
        thread::sleep(one_second);
    }
}
