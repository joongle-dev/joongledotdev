use std::mem::MaybeUninit;
use super::marker::{True, IsPowerOfTwo};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct OutOfSpaceError;
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct OutOfElementError;

pub struct FixedRingBuffer<T, const C: usize> where IsPowerOfTwo<C>: True {
    head: usize,
    len: usize,
    buffer: [MaybeUninit<T>; C],
}

impl<T, const C: usize> FixedRingBuffer<T, C> where IsPowerOfTwo<C>: True {
    const UNINIT: MaybeUninit<T> = MaybeUninit::uninit(); //Constant used to initialize buffer.
    pub fn new() -> Self {
        Self {
            head: 0,
            len: 0,
            buffer: [Self::UNINIT; C],
        }
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn push_front(&mut self, val: T) -> Result<(), OutOfSpaceError> {
        if self.len == C {
            return Err(OutOfSpaceError)
        }
        self.len += 1;
        self.head = (self.head + C - 1) & (C - 1);
        self.push(self.head, val);
        Ok(())
    }
    pub fn pop_front(&mut self) -> Result<T, OutOfElementError> {
        if self.len == 0 {
            return Err(OutOfElementError)
        }
        let val = self.pop(self.head);
        self.len -= 1;
        self.head = (self.head + 1) & (C - 1);
        Ok(val)
    }
    pub fn push_back(&mut self, val: T) -> Result<(), OutOfSpaceError> {
        if self.len == C {
            return Err(OutOfSpaceError)
        }
        self.push((self.head + self.len) & (C - 1), val);
        self.len += 1;
        Ok(())
    }
    pub fn pop_back(&mut self) -> Result<T, OutOfElementError> {
        if self.len == 0 {
            return Err(OutOfElementError)
        }
        self.len -= 1;
        Ok(self.pop((self.head + self.len) & (C - 1)))
    }
    fn push(&mut self, idx: usize, val: T) {
        self.buffer[idx] = MaybeUninit::new(val);
    }
    fn pop(&mut self, idx: usize) -> T {
        unsafe { std::mem::replace(&mut self.buffer[idx], Self::UNINIT).assume_init() }
    }
}

impl<T, const C: usize> Drop for FixedRingBuffer<T, C> where IsPowerOfTwo<C>: True {
    fn drop(&mut self) {
        while self.len > 0 {
            let _ = self.pop_back();
        }
    }
}