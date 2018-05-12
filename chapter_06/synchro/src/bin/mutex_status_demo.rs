use std::sync::{Arc, Mutex};
use std::{thread, time};

const THRS: usize = 4;
static mut COUNTS: &'static mut [u64] = &mut [0; THRS];
static mut STATUS: &'static mut [bool] = &mut [false; THRS];

fn worker(id: usize, gate: Arc<Mutex<()>>) -> () {
    unsafe {
        loop {
            let guard = gate.lock().unwrap();
            STATUS[id] = true;
            COUNTS[id] += 1;
            STATUS[id] = false;
            drop(guard);
        }
    }
}

fn main() {
    let mutex = Arc::new(Mutex::new(()));

    for i in 0..THRS {
        let mutex = Arc::clone(&mutex);
        thread::spawn(move || worker(i, mutex));
    }

    let mut counts: [u64; THRS] = [0; THRS];
    loop {
        unsafe {
            thread::sleep(time::Duration::from_millis(1_000));
            print!("|");
            for i in 0..THRS {
                print!(" {:>5}; {:010}/sec |", STATUS[i], COUNTS[i] - counts[i]);
                counts[i] = COUNTS[i];
            }
            println!();
        }
    }
}
