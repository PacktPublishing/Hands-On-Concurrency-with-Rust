use std::thread;
use std::sync::mpsc;

fn main() {
    let total_readers = 5;
    let mut sends = Vec::with_capacity(total_readers);

    let mut reader_jhs = Vec::with_capacity(total_readers);
    for _ in 0..total_readers {
        let (snd, rcv) = mpsc::sync_channel(64);
        sends.push(snd);
        reader_jhs.push(thread::spawn(move || {
            let mut total_zeros = 0;
            let mut seen_values = 0;
            for v in rcv {
                seen_values += 1;
                if v == 0 {
                    total_zeros += 1;
                }
                if total_zeros >= 100 {
                    break;
                }
            }
            seen_values
        }));
    }

    {
        let mut loops = 0;
        let mut cur: u16 = 0;
        while loops < 100 {
            cur = cur.wrapping_add(1);
            for snd in &sends {
                snd.send(cur).expect("failed to send");
            }
            if cur == 0 {
                loops += 1;
            }
        }
    }

    for jh in reader_jhs {
        println!("{:?}", jh.join().unwrap());
    }
}
