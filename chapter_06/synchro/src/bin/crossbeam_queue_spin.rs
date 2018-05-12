extern crate crossbeam;

use crossbeam::sync::MsQueue;
use std::sync::Arc;
use std::thread;

fn main() {
    let q = Arc::new(MsQueue::new());

    let mut jhs = Vec::new();

    for _ in 0..4 {
        let q = Arc::clone(&q);
        jhs.push(thread::spawn(move || {
            let mut i = 0;
            loop {
                q.push(i);
                i += 1;
                q.pop();
            }
        }))
    }

    for jh in jhs {
        jh.join().unwrap();
    }
}
