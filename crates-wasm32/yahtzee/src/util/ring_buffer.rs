use core::mem::MaybeUninit;
use super::marker::{True, IsPowerOfTwo};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct OutOfSpaceError;

pub struct FixedRingBuffer<T, const C: usize> where IsPowerOfTwo<C>: True {
    buffer: [MaybeUninit<T>; C],
    head: usize,
    len: usize,
}

impl<T, const C: usize> FixedRingBuffer<T, C> where IsPowerOfTwo<C>: True {
    const UNINIT: MaybeUninit<T> = MaybeUninit::uninit(); //Constant used to initialize buffer.
    pub fn new() -> Self {
        Self {
            buffer: [Self::UNINIT; C],
            head: 0,
            len: 0,
        }
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn try_push_front(&mut self, val: T) -> Result<(), OutOfSpaceError> {
        if self.len == C {
            return Err(OutOfSpaceError)
        }
        self.len += 1;
        self.head = (self.head + C - 1) & (C - 1);
        self.buffer[self.head] = MaybeUninit::new(val);
        Ok(())
    }
    pub fn push_front(&mut self, val: T) {
        self.try_push_front(val).unwrap()
    }
    pub fn pop_front(&mut self) -> Option<T> {
        if self.len == 0 {
            return None
        }
        let val = unsafe { self.buffer[self.head].assume_init_read() };
        self.len -= 1;
        self.head = (self.head + 1) & (C - 1);
        Some(val)
    }
    pub fn try_push_back(&mut self, val: T) -> Result<(), OutOfSpaceError> {
        if self.len == C {
            return Err(OutOfSpaceError)
        }
        self.buffer[self.head + self.len] = MaybeUninit::new(val);
        self.len += 1;
        Ok(())
    }
    pub fn push_back(&mut self, val: T) {
        self.try_push_back(val).unwrap()
    }
    pub fn pop_back(&mut self) -> Option<T> {
        if self.len == 0 {
            return None
        }
        self.len -= 1;
        Some(unsafe { self.buffer[(self.head + self.len) & (C - 1)].assume_init_read() })
    }
}

impl<T, const C: usize> Drop for FixedRingBuffer<T, C> where IsPowerOfTwo<C>: True {
    fn drop(&mut self) {
        while self.len > 0 {
            let _ = self.pop_back();
        }
    }
}