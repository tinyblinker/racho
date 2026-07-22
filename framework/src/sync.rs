//! only guaranteed in uniprocessor env
use core::cell::{RefCell, RefMut};

pub struct UPSafeCell<T> {
    inner: RefCell<T>,
}

unsafe impl<T> Sync for UPSafeCell<T> {}

impl<T> UPSafeCell<T> {
    /// only used in the uniprocessor please
    pub fn new(val: T) -> Self {
        Self {
            inner: RefCell::new(val),
        }
    }
    /// exclusive access inner data in UPSafeCell. Panic if the data has been borrowed
    pub fn exclusive_access(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}
