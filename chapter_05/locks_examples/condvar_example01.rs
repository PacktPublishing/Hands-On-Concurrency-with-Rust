use std::thread;
use std::sync::{Arc, Condvar, Mutex};

fn main() {
    let total_readers = 5;
    let mutcond: Arc<(Mutex<(bool, u16)>, Condvar)> =
        Arc::new((Mutex::new((false, 0)), Condvar::new()));

    let mut reader_jhs = Vec::with_capacity(total_readers);
    for _ in 0..total_readers {
        let mutcond = Arc::clone(&mutcond);
        reader_jhs.push(thread::spawn(move || {
            let mut total_zeros = 0;
            let mut total_wakes = 0;
            let &(ref mtx, ref cnd) = &*mutcond;

            while total_zeros < 100 {
                let mut guard = mtx.lock().unwrap();
                while !guard.0 {
                    guard = cnd.wait(guard).unwrap();
                }
                guard.0 = false;

                total_wakes += 1;
                if guard.1 == 0 {
                    total_zeros += 1;
                }
            }
            total_wakes
        }));
    }

    let _ = thread::spawn(move || {
        let &(ref mtx, ref cnd) = &*mutcond;
        loop {
            let mut guard = mtx.lock().unwrap();
            guard.1 = guard.1.wrapping_add(1);
            guard.0 = true;
            cnd.notify_all();
        }
    });

    for jh in reader_jhs {
        println!("{:?}", jh.join().unwrap());
    }
}
