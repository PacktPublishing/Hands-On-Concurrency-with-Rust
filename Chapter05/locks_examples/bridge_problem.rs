use std::thread;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
enum Bridge {
    Empty,
    Left(u8),
    Right(u8),
}

fn main() {
    let rope = Arc::new(Mutex::new(Bridge::Empty));

    let lhs_rope = Arc::clone(&rope);
    let lhs = thread::spawn(move || {
        let rope = lhs_rope;
        loop {
            let mut guard = rope.lock().unwrap();
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
                    *guard = Bridge::Left(i - 1);
                }
            }
        }
    });

    let rhs = thread::spawn(move || loop {
        let mut guard = rope.lock().unwrap();
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
                *guard = Bridge::Right(i - 1);
            }
        }
    });

    rhs.join().unwrap();
    lhs.join().unwrap();
}
