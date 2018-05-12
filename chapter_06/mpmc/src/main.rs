#[macro_use]
extern crate lazy_static;

use std::thread;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

lazy_static! {
static ref X: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
static ref Y: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
static ref Z: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
}

fn write_x() {
    X.store(true, Ordering::SeqCst);
}

fn write_y() {
    Y.store(true, Ordering::SeqCst);
}

fn read_x_then_y() {
    while !X.load(Ordering::SeqCst) {}
    if Y.load(Ordering::SeqCst) {
        Z.fetch_add(1, Ordering::Relaxed);
    }
}

fn read_y_then_x() {
    while !Y.load(Ordering::SeqCst) {}
    if X.load(Ordering::SeqCst) {
        Z.fetch_add(1, Ordering::Relaxed);
    }
}

fn main() {
    let mut jhs = Vec::new();
    jhs.push(thread::spawn(write_x)); // a
    jhs.push(thread::spawn(write_y)); // b
    jhs.push(thread::spawn(read_x_then_y)); // c
    jhs.push(thread::spawn(read_y_then_x)); // d
    for jh in jhs {
        jh.join().unwrap();
    }
    assert!(Z.load(Ordering::Relaxed) != 0);
}
