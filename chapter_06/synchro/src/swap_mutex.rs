use std::ops::{Deref, DerefMut};
use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};

unsafe impl<T: Send> Send for SwapMutex<T> {}
unsafe impl<T: Send> Sync for SwapMutex<T> {}

pub struct SwapMutex<T> {
    locked: AtomicBool,
    data: *mut T,
}

impl<T> SwapMutex<T> {
    pub fn new(t: T) -> Self {
        let boxed_data = Box::new(t);
        SwapMutex {
            locked: AtomicBool::new(false),
            data: Box::into_raw(boxed_data),
        }
    }

    pub fn lock(&self) -> SwapMutexGuard<T> {
        while self.locked.swap(true, Ordering::AcqRel) {
            thread::yield_now();
        }
        SwapMutexGuard::new(self)
    }

    fn unlock(&self) -> () {
        assert!(self.locked.load(Ordering::Relaxed) == true);
        self.locked.store(false, Ordering::Release);
    }
}

impl<T> Drop for SwapMutex<T> {
    fn drop(&mut self) {
        let data = unsafe { Box::from_raw(self.data) };
        drop(data);
    }
}

pub struct SwapMutexGuard<'a, T: 'a> {
    __lock: &'a SwapMutex<T>,
}

impl<'a, T> SwapMutexGuard<'a, T> {
    fn new(lock: &'a SwapMutex<T>) -> SwapMutexGuard<'a, T> {
        SwapMutexGuard { __lock: lock }
    }
}

impl<'a, T> Deref for SwapMutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.__lock.data }
    }
}

impl<'a, T> DerefMut for SwapMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.__lock.data }
    }
}

impl<'a, T> Drop for SwapMutexGuard<'a, T> {
    #[inline]
    fn drop(&mut self) {
        self.__lock.unlock();
    }
}
