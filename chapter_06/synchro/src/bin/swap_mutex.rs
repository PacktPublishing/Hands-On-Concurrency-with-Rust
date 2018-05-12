extern crate synchro;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use synchro::SwapMutex;
use std::{thread, time};

#[derive(Debug)]
enum Bridge {
    Empty,
    Left(u8),
    Right(u8),
}

static LHS_TRANSFERS: AtomicUsize = AtomicUsize::new(0);
static RHS_TRANSFERS: AtomicUsize = AtomicUsize::new(0);

fn lhs(rope: Arc<SwapMutex<Bridge>>) -> () {
    loop {
        let mut guard = rope.lock();
        match *guard {
            Bridge::Empty => {
                *guard = Bridge::Right(1);
            }
            Bridge::Right(i) => {
                if i < 5 {
                    *guard = Bridge::Right(i + 1);
                }
            }
            Bridge::Left(0) => {
                *guard = Bridge::Empty;
            }
            Bridge::Left(i) => {
                LHS_TRANSFERS.fetch_add(1, Ordering::Relaxed);
                *guard = Bridge::Left(i - 1);
            }
        }
    }
}

fn rhs(rope: Arc<SwapMutex<Bridge>>) -> () {
    loop {
        let mut guard = rope.lock();
        match *guard {
            Bridge::Empty => {
                *guard = Bridge::Left(1);
            }
            Bridge::Left(i) => {
                if i < 5 {
                    *guard = Bridge::Left(i + 1);
                }
            }
            Bridge::Right(0) => {
                *guard = Bridge::Empty;
            }
            Bridge::Right(i) => {
                RHS_TRANSFERS.fetch_add(1, Ordering::Relaxed);
                *guard = Bridge::Right(i - 1);
            }
        }
    }
}

fn main() {
    let mtx: Arc<SwapMutex<Bridge>> = Arc::new(SwapMutex::new(Bridge::Empty));

    let lhs_mtx = Arc::clone(&mtx);
    let _lhs = thread::spawn(move || lhs(lhs_mtx));
    let _rhs = thread::spawn(move || rhs(mtx));

    let one_second = time::Duration::from_millis(1_000);
    loop {
        thread::sleep(one_second);
        println!(
            "Transfers per second:\n    LHS: {}\n    RHS: {}",
            LHS_TRANSFERS.swap(0, Ordering::Relaxed),
            RHS_TRANSFERS.swap(0, Ordering::Relaxed)
        );
    }
}
