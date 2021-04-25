// use crate::error::Error;
use std::slice::Iter;

#[derive(Debug)]
pub struct Stack<T> {
    st: Vec<T>,
    lock: bool,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Self {
            st: Vec::<T>::new(),
            lock: false,
        }
    }
    pub fn push(&mut self, value: T) {
        self.st.push(value);
    }
    pub fn pop(&mut self) -> Option<T> {
        self.st.pop()
    }
    pub fn iter(&mut self) -> Iter<T> {
        self.st.iter()
    }
    pub fn len(&mut self) -> usize {
        self.st.len()
    }
    // pub fn append(&mut self, other: &mut Vec<T>) {
    //     self.st.append(other)
    // }
    pub fn clear(&mut self) {
        self.st.clear();
    }
    // pub fn lock(&mut self) -> Result<(), Error> {
    //     if self.lock {
    //         Err(Error::StackLocked)
    //     } else {
    //         Ok(())
    //     }
    // }
    // pub fn unlock(&mut self) -> Result<(), Error> {
    //     Ok(self.lock = false)
    // }
    // pub fn is_locked(&self) -> bool {
    //     self.lock
    // }
}
