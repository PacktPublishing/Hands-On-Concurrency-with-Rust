use std::ptr::null_mut;
use std::sync::atomic::{AtomicPtr, Ordering};

unsafe impl<T: Send> Send for Queue<T> {}
unsafe impl<T: Send> Sync for Queue<T> {}

struct Node<T> {
    value: *const T,
    next: AtomicPtr<Node<T>>,
}

impl<T> Default for Node<T> {
    fn default() -> Self {
        Node {
            value: null_mut(),
            next: AtomicPtr::default(),
        }
    }
}

impl<T> Node<T> {
    fn new(val: T) -> Self {
        Node {
            value: Box::into_raw(Box::new(val)),
            next: AtomicPtr::default(),
        }
    }
}

struct InnerQueue<T> {
    head: AtomicPtr<Node<T>>,
    tail: AtomicPtr<Node<T>>,
}

impl<T> InnerQueue<T> {
    pub fn new() -> Self {
        let node = Box::into_raw(Box::new(Node::default()));
        InnerQueue {
            head: AtomicPtr::new(node),
            tail: AtomicPtr::new(node),
        }
    }

    pub unsafe fn enq(&mut self, val: T) -> () {
        let node = Box::new(Node::new(val));
        let node: *mut Node<T> = Box::into_raw(node);

        loop {
            let tail: *mut Node<T> = self.tail.load(Ordering::Acquire);
            let next: *mut Node<T> = (*tail).next.load(Ordering::Relaxed);
            if tail == self.tail.load(Ordering::Relaxed) {
                if next.is_null() {
                    if (*tail).next.compare_and_swap(next, node, Ordering::Relaxed) == next {
                        self.tail.compare_and_swap(tail, node, Ordering::Release);
                        return;
                    }
                }
            } else {
                self.tail.compare_and_swap(tail, next, Ordering::Release);
            }
        }
    }

    pub unsafe fn deq(&mut self) -> Option<T> {
        let mut head: *mut Node<T>;
        let value: T;
        loop {
            head = self.head.load(Ordering::Acquire);
            let tail: *mut Node<T> = self.tail.load(Ordering::Relaxed);
            let next: *mut Node<T> = (*head).next.load(Ordering::Relaxed);
            if head == self.head.load(Ordering::Relaxed) {
                if head == tail {
                    if next.is_null() {
                        return None;
                    }
                    self.tail.compare_and_swap(tail, next, Ordering::Relaxed);
                } else {
                    let val: *mut T = (*next).value as *mut T;
                    if self.head.compare_and_swap(head, next, Ordering::Release) == head {
                        value = *Box::from_raw(val);
                        break;
                    }
                }
            }
        }
        // let head: Node<T> = *Box::from_raw(head);
        // drop(head);
        Some(value)
    }
}

pub struct Queue<T> {
    inner: *mut InnerQueue<T>,
}

impl<T> Clone for Queue<T> {
    fn clone(&self) -> Queue<T> {
        Queue { inner: self.inner }
    }
}

impl<T> Queue<T> {
    pub fn new() -> Self {
        Queue {
            inner: Box::into_raw(Box::new(InnerQueue::new())),
        }
    }

    pub fn enq(&self, val: T) -> () {
        unsafe { (*self.inner).enq(val) }
    }

    pub fn deq(&self) -> Option<T> {
        unsafe { (*self.inner).deq() }
    }
}

#[cfg(test)]
mod test {
    extern crate quickcheck;

    use super::*;
    use std::collections::VecDeque;
    use std::sync::atomic::AtomicUsize;
    use std::thread;
    use std::sync::Arc;
    use self::quickcheck::{Arbitrary, Gen, QuickCheck, TestResult};

    #[derive(Clone, Debug)]
    enum Op {
        Enq(u32),
        Deq,
    }

    impl Arbitrary for Op {
        fn arbitrary<G>(g: &mut G) -> Self
        where
            G: Gen,
        {
            let i: usize = g.gen_range(0, 2);
            match i {
                0 => Op::Enq(g.gen()),
                _ => Op::Deq,
            }
        }
    }

    #[test]
    fn sequential() {
        fn inner(ops: Vec<Op>) -> TestResult {
            let mut vd = VecDeque::new();
            let q = Queue::new();

            for op in ops {
                match op {
                    Op::Enq(v) => {
                        vd.push_back(v);
                        q.enq(v);
                    }
                    Op::Deq => {
                        assert_eq!(vd.pop_front(), q.deq());
                    }
                }
            }
            TestResult::passed()
        }
        QuickCheck::new().quickcheck(inner as fn(Vec<Op>) -> TestResult);
    }

    fn parallel_exp(total: usize, enqs: u8, deqs: u8) -> bool {
        let q = Queue::new();
        let total_expected = total * (enqs as usize);
        let total_retrieved = Arc::new(AtomicUsize::new(0));

        let mut ejhs = Vec::new();
        for _ in 0..enqs {
            let mut q = q.clone();
            ejhs.push(
                thread::Builder::new()
                    .spawn(move || {
                        for i in 0..total {
                            q.enq(i);
                        }
                    })
                    .unwrap(),
            );
        }

        let mut djhs = Vec::new();
        for _ in 0..deqs {
            let mut q = q.clone();
            let total_retrieved = Arc::clone(&total_retrieved);
            djhs.push(
                thread::Builder::new()
                    .spawn(move || {
                        while total_retrieved.load(Ordering::Relaxed) != total_expected {
                            if q.deq().is_some() {
                                total_retrieved.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    })
                    .unwrap(),
            );
        }

        for jh in ejhs {
            jh.join().unwrap();
        }
        for jh in djhs {
            jh.join().unwrap();
        }

        assert_eq!(total_retrieved.load(Ordering::Relaxed), total_expected);
        true
    }

    #[test]
    fn repeated() {
        for i in 0..10_000 {
            println!("{}", i);
            parallel_exp(73, 2, 2);
        }
    }

    #[test]
    fn parallel() {
        fn inner(total: usize, enqs: u8, deqs: u8) -> TestResult {
            if enqs == 0 || deqs == 0 {
                TestResult::discard()
            } else {
                TestResult::from_bool(parallel_exp(total, enqs, deqs))
            }
        }
        QuickCheck::new().quickcheck(inner as fn(usize, u8, u8) -> TestResult);
    }
}
