use std::sync::atomic::{fence, AtomicPtr, AtomicUsize, Ordering};
use std::{mem, ptr};

unsafe impl<T: Send> Send for Stack<T> {}
unsafe impl<T: Send> Sync for Stack<T> {}

struct Node<T> {
    references: AtomicUsize,
    next: *mut Node<T>,
    data: Option<T>,
}

impl<T> Node<T> {
    fn new(t: T) -> Self {
        Node {
            references: AtomicUsize::new(1),
            next: ptr::null_mut(),
            data: Some(t),
        }
    }
}

pub struct Stack<T> {
    head: AtomicPtr<Node<T>>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Stack {
            head: AtomicPtr::default(),
        }
    }

    pub fn pop(&self) -> Option<T> {
        loop {
            let head: *mut Node<T> = self.head.load(Ordering::Relaxed);

            if head.is_null() {
                return None;
            }
            let next: *mut Node<T> = unsafe { (*head).next };

            if self.head.compare_and_swap(head, next, Ordering::Relaxed) == head {
                let mut head: Box<Node<T>> = unsafe { Box::from_raw(head) };
                let data: Option<T> = mem::replace(&mut (*head).data, None);
                unsafe {
                    assert_eq!(
                        (*(*head).next).references.fetch_sub(1, Ordering::Release),
                        2
                    );
                }
                drop(head);
                return data;
            }
        }
    }

    pub fn push(&self, t: T) -> () {
        let node: *mut Node<T> = Box::into_raw(Box::new(Node::new(t)));
        loop {
            let head = self.head.load(Ordering::Relaxed);
            unsafe {
                (*node).next = head;
            }

            fence(Ordering::Acquire);
            if self.head.compare_and_swap(head, node, Ordering::Release) == head {
                // node is now self.head
                // head is now self.head.next
                if !head.is_null() {
                    unsafe {
                        // assert_eq!(1, (*head).references.fetch_add(1, Ordering::Release));
                        assert_eq!(1, (*head).references.fetch_add(1, Ordering::Release));
                    }
                }
                break;
            }
        }
    }
}
