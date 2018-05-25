extern crate num_cpus;
extern crate treiber_stacks;

use std::sync::Arc;
use std::thread;
use treiber_stacks::refcount::Stack;

fn main() {
    let stk: Arc<Stack<(u64, u64, u64)>> = Arc::new(Stack::new());

    let mut jhs = Vec::new();

    for _ in 0..num_cpus::get() {
        let stk = Arc::clone(&stk);
        jhs.push(thread::spawn(move || {
            let mut i = 0;
            loop {
                stk.push((i, i, i));
                i += 1;
                stk.pop();
            }
        }))
    }

    for jh in jhs {
        jh.join().unwrap();
    }
}
