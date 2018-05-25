extern crate hopper;
extern crate tempdir;

use std::{mem, thread};

fn writer(mut chan: hopper::Sender<u32>) -> () {
    let mut cur: u32 = 0;
    while let Ok(()) = chan.send(cur) {
        cur = cur.wrapping_add(1);
    }
}

fn reader(read_limit: usize, mut chan: hopper::Receiver<u32>) -> () {
    let mut cur: u32 = 0;
    let mut iter = chan.iter();
    while (cur as usize) < read_limit {
        let num = iter.next().unwrap();
        assert_eq!(num, cur);
        cur = cur.wrapping_add(1);
    }
}

fn main() {
    let read_limit = 1_000_000;
    let in_memory_capacity = mem::size_of::<u32>() * 10;
    let on_disk_capacity = mem::size_of::<u32>() * 100_000;

    let dir = tempdir::TempDir::new("queue_root").unwrap();
    let (snd, rcv) = hopper::channel_with_explicit_capacity::<u32>(
        "example",
        dir.path(),
        in_memory_capacity,
        on_disk_capacity,
        1,
    ).unwrap();

    let reader_jh = thread::spawn(move || {
        reader(read_limit, rcv);
    });
    let _writer_jh = thread::spawn(move || {
        writer(snd);
    });

    reader_jh.join().unwrap();
}
