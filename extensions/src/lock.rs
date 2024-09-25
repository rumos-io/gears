use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

const POISONED_LOCK_MSG: &str = "poisoned lock";

pub trait AcquireRwLock {
    type Output;

    fn acquire_read(&self) -> RwLockReadGuard<'_, Self::Output>;
    fn acquire_write(&self) -> RwLockWriteGuard<'_, Self::Output>;
}

impl<T> AcquireRwLock for RwLock<T> {
    type Output = T;

    fn acquire_read(&self) -> RwLockReadGuard<'_, Self::Output> {
        self.read().expect(POISONED_LOCK_MSG)
    }

    fn acquire_write(&self) -> RwLockWriteGuard<'_, Self::Output> {
        self.write().expect(POISONED_LOCK_MSG)
    }
}
