use std::{mem, thread};
use std::sync::{Arc, Mutex};

struct Ring {
    size: usize,
    data: Vec<Option<u32>>,
}

impl Ring {
    fn with_capacity(capacity: usize) -> Ring {
        let mut data: Vec<Option<u32>> = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            data.push(None);
        }
        Ring {
            size: 0,
            data: data,
        }
    }

    fn capacity(&self) -> usize {
        self.data.capacity()
    }

    fn is_full(&self) -> bool {
        self.size == self.data.capacity()
    }

    fn emplace(&mut self, offset: usize, val: u32) -> Option<u32> {
        self.size += 1;
        let res = mem::replace(&mut self.data[offset], Some(val));
        res
    }

    fn displace(&mut self, offset: usize) -> Option<u32> {
        let res = mem::replace(&mut self.data[offset], None);
        if res.is_some() {
            self.size -= 1;
        }
        res
    }
}

fn writer(ring_lk: Arc<Mutex<Ring>>) -> () {
    let mut offset: usize = 0;
    let mut cur: u32 = 0;
    loop {
        let mut ring = ring_lk.lock().unwrap();
        if !ring.is_full() {
            assert!(ring.emplace(offset, cur).is_none());
            cur = cur.wrapping_add(1);
            offset += 1;
            offset %= ring.capacity();
        } else {
            thread::yield_now();
        }
    }
}

fn reader(read_limit: usize, ring_lk: Arc<Mutex<Ring>>) -> () {
    let mut offset: usize = 0;
    let mut cur: u32 = 0;
    while (cur as usize) < read_limit {
        let mut ring = ring_lk.lock().unwrap();
        if let Some(num) = ring.displace(offset) {
            assert_eq!(num, cur);
            cur = cur.wrapping_add(1);
            offset += 1;
            offset %= ring.capacity();
        } else {
            drop(ring);
            thread::yield_now();
        }
    }
}

fn main() {
    let capacity = 10;
    let read_limit = 1_000_000;
    let ring = Arc::new(Mutex::new(Ring::with_capacity(capacity)));

    let reader_ring = Arc::clone(&ring);
    let reader_jh = thread::spawn(move || {
        reader(read_limit, reader_ring);
    });
    let _writer_jh = thread::spawn(move || {
        writer(ring);
    });

    reader_jh.join().unwrap();
}
