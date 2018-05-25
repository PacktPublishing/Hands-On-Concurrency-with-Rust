use crossbeam::sync::MsQueue;

unsafe impl Send for Semaphore {}
unsafe impl Sync for Semaphore {}

pub struct Semaphore {
    capacity: MsQueue<()>,
}

impl Semaphore {
    pub fn new(capacity: usize) -> Self {
        let q = MsQueue::new();
        for _ in 0..capacity {
            q.push(());
        }
        Semaphore { capacity: q }
    }

    pub fn wait(&self) -> () {
        self.capacity.pop();
    }

    pub fn signal(&self) -> () {
        self.capacity.push(());
    }
}
