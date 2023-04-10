use crate::proc::Proc;
use core::ptr::null_mut;

pub struct Queue {
    front: *mut Proc,
    back: *mut Proc,
}

impl Queue {
    pub fn push_front(&mut self, n: &mut Proc) {
        unsafe {
            let n = n as *mut Proc;
            (*n).data.next = self.front;
            (*n).data.prev = null_mut();
            if self.front != null_mut() {
                (*self.front).data.prev = n;
            } else {
                self.back = n;
            }
            self.front = n;
        }
    }
    pub unsafe fn remove(&mut self, n: *mut Proc) {
        if (*n).data.next != null_mut() {
            (*(*n).data.next).data.prev = (*n).data.prev;
        } else {
            self.back = (*n).data.prev;
        }
        if (*n).data.prev != null_mut() {
            (*(*n).data.prev).data.next = (*n).data.next;
        } else {
            self.front = (*n).data.next;
        }
    }
    pub fn pop_back(&mut self) -> Option<&mut Proc> {
        let b = self.back;
        if b == null_mut() {
            return None;
        }
        unsafe {
            self.remove(b);
            Some(&mut *b)
        }
    }
}
