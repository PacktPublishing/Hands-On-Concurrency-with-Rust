// start snippet prelim
use std::{mem, thread};
use std::ops::{Deref, DerefMut};

unsafe impl Send for Ring {}
unsafe impl Sync for Ring {}

struct InnerRing {
    capacity: isize,
    size: usize,
    data: *mut Option<u32>,
}

#[derive(Clone)]
struct Ring {
    inner: *mut InnerRing,
}
// end snippet prelim

// start snippet derefs
impl Deref for Ring {
    type Target = InnerRing;

    fn deref(&self) -> &InnerRing {
        unsafe { &*self.inner }
    }
}

impl DerefMut for Ring {
    fn deref_mut(&mut self) -> &mut InnerRing {
        unsafe { &mut *self.inner }
    }
}
// end snippet derefs

// start snippet implement
impl Ring {
    fn with_capacity(capacity: usize) -> Ring {
        let mut data: Vec<Option<u32>> = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            data.push(None);
        }
        let raw_data = (&mut data).as_mut_ptr();
        mem::forget(data);
        let inner_ring = Box::new(InnerRing {
            capacity: capacity as isize,
            size: 0,
            data: raw_data,
        });

        Ring {
            inner: Box::into_raw(inner_ring),
        }
    }
}
// end snippet implement

// start snippet writer
fn writer(mut ring: Ring) -> () {
    let mut offset: isize = 0;
    let mut cur: u32 = 0;
    loop {
        unsafe {
            if (*ring).size != ((*ring).capacity as usize) {
                assert!(mem::replace(&mut *(*ring).data.offset(offset), Some(cur)).is_none());
                (*ring).size += 1;
                cur = cur.wrapping_add(1);
                offset += 1;
                offset %= (*ring).capacity;
            } else {
                thread::yield_now();
            }
        }
    }
}
// end snippet writer

// start snippet reader
fn reader(mut ring: Ring) -> () {
    let mut offset: isize = 0;
    let mut cur: u32 = 0;
    while cur < 1_000 {
        unsafe {
            if let Some(num) = mem::replace(&mut *(*ring).data.offset(offset), None) {
                assert_eq!(num, cur);
                (*ring).size -= 1;
                cur = cur.wrapping_add(1);
                offset += 1;
                offset %= (*ring).capacity;
            } else {
                thread::yield_now();
            }
        }
    }
}
// end snippet reader

// start snippet main
fn main() {
    let capacity = 10;
    let ring = Ring::with_capacity(capacity);

    let reader_ring = ring.clone();
    let reader_jh = thread::spawn(move || {
        reader(reader_ring);
    });
    let _writer_jh = thread::spawn(move || {
        writer(ring);
    });

    reader_jh.join().unwrap();
}
// end snippet main
