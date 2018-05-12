extern crate synchro;

use synchro::Queue;
use std::thread;

fn main() {
    let q = Queue::new();

    let mut jhs = Vec::new();

    for _ in 0..4 {
        let eq = q.clone();
        jhs.push(thread::spawn(move || {
            let mut i = 0;
            loop {
                eq.enq(i);
                i += 1;
                eq.deq();
            }
        }))
    }

    for jh in jhs {
        jh.join().unwrap();
    }
}
