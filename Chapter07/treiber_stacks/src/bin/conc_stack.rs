extern crate conc;
#[macro_use]
extern crate lazy_static;
extern crate num_cpus;
extern crate quantiles;

use conc::sync::Treiber;
use quantiles::ckms::CKMS;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{thread, time};

lazy_static! {
    static ref WORKERS: AtomicUsize = AtomicUsize::new(0);
    static ref COUNT: AtomicUsize = AtomicUsize::new(0);
}
static MAX_I: u32 = 67_108_864; // 2 ** 26

fn main() {
    let stk: Arc<Treiber<(u64, u64, u64)>> = Arc::new(Treiber::new());

    let mut jhs = Vec::new();

    let cpus = num_cpus::get();
    WORKERS.store(cpus, Ordering::Release);

    for _ in 0..cpus {
        let stk = Arc::clone(&stk);
        jhs.push(thread::spawn(move || {
            for i in 0..MAX_I {
                stk.push((i as u64, i as u64, i as u64));
                stk.pop();
                COUNT.fetch_add(1, Ordering::Relaxed);
            }
            WORKERS.fetch_sub(1, Ordering::Relaxed)
        }))
    }

    let one_second = time::Duration::from_millis(1_000);
    let mut iter = 0;
    let mut cycles: CKMS<u32> = CKMS::new(0.001);
    while WORKERS.load(Ordering::Relaxed) != 0 {
        let count = COUNT.swap(0, Ordering::Relaxed);
        cycles.insert((count / cpus) as u32);
        println!(
            "CYCLES PER SECOND({}):\n  25th: {}\n  50th: {}\n  75th: {}\n  90th: {}\n  max:  {}\n",
            iter,
            cycles.query(0.25).unwrap().1,
            cycles.query(0.50).unwrap().1,
            cycles.query(0.75).unwrap().1,
            cycles.query(0.90).unwrap().1,
            cycles.query(1.0).unwrap().1
        );
        thread::sleep(one_second);
        iter += 1;
    }

    for jh in jhs {
        jh.join().unwrap();
    }
}
