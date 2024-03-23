// a linked list without dynamic memory allocation
// internal nodes are just raw pointers pointing to each other 
use core::ptr::null_mut;

#[derive(Clone, Copy, Debug)]
pub struct LinkedList {
    pub head: *mut usize
}

impl LinkedList {
    pub const fn new() -> Self {
        LinkedList { head: null_mut() }
    }

    pub fn push(&mut self, elem: *mut usize) {
        unsafe {
            *elem = self.head as usize;
            self.head = elem;
        }
    }

    pub fn empty(&self) -> bool {
        self.head.is_null()
    }

    pub fn pop(&mut self) -> Option<*mut usize> {
        if self.head.is_null() {
            None
        } else {
            let head = self.head;
            unsafe {
                self.head = *head as *mut usize;
            }
            Some(head)
        }
    }

    pub fn iter(&mut self) -> LinkedListIter {
        LinkedListIter {
            prev: &mut self.head as *mut *mut usize as *mut usize,
            ptr: self.head
        }
    }
}

pub struct LinkedListIter {
    prev: *mut usize,
    ptr: *mut usize
}

impl LinkedListIter {
    pub fn get(&self) -> *mut usize {
        self.ptr
    }

    pub fn pop(self) {
        unsafe {
            *self.prev = *self.ptr;
        }
    }
}

impl Iterator for LinkedListIter {
    type Item = LinkedListIter;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr.is_null() {
            None
        } else {
            let ptr = self.ptr;
            let res = LinkedListIter {
                prev: self.prev,
                ptr: self.ptr
            };
            unsafe {
                self.ptr = *ptr as *mut usize;
                self.prev = ptr;
            }
            Some(res)
        }
    }
}
