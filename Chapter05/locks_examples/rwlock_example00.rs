use std::thread;
use std::sync::{Arc, RwLock};

fn main() {
    let resource: Arc<RwLock<u16>> = Arc::new(RwLock::new(0));

    let total_readers = 5;

    let mut reader_jhs = Vec::with_capacity(total_readers);
    for _ in 0..total_readers {
        let resource = Arc::clone(&resource);
        reader_jhs.push(thread::spawn(move || {
            let mut total_lock_success = 0;
            let mut total_lock_failure = 0;
            let mut total_zeros = 0;
            while total_zeros < 100 {
                match resource.try_read() {
                    Ok(guard) => {
                        total_lock_success += 1;
                        if *guard == 0 {
                            total_zeros += 1;
                        }
                    }
                    Err(_) => {
                        total_lock_failure += 1;
                    }
                }
            }
            (total_lock_failure, total_lock_success)
        }));
    }

    {
        let mut loops = 0;
        while loops < 100 {
            let mut guard = resource.write().unwrap();
            *guard = guard.wrapping_add(1);
            if *guard == 0 {
                loops += 1;
            }
        }
    }

    for jh in reader_jhs {
        println!("{:?}", jh.join().unwrap());
    }
}
