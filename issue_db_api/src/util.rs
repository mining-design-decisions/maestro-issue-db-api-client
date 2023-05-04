use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::RwLock;
use crate::errors::{APIResult};


macro_rules! initialize_lazy_field {
    ($e:expr) => {
        {
            let value: Option<_> = $e;
            let lazy = lazy_init::Lazy::new();
            if let Some(x) = value {
                let _ = lazy.get_or_create(|| x);
            }
            lazy
        }
    }
}


pub(crate) use initialize_lazy_field;



#[allow(unused)]
#[derive(Debug)]
pub struct CacheLockError {}

impl std::fmt::Display for CacheLockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Poisoned lock in cache. Cannot recover.")
    }
}

impl std::error::Error for CacheLockError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}


#[derive(Debug)]
pub struct CacheContainer<T: Clone> {
    value: RwLock<Option<T>>,
    dirty: AtomicBool
}


impl<T: Clone> CacheContainer<T> {
    pub fn new(initial: Option<T>) -> Self {
        CacheContainer{
            value: RwLock::new(initial),
            dirty: AtomicBool::new(true)
        }
    }

    pub fn get<F: FnOnce() -> APIResult<T>>(&self, f: F) -> APIResult<T> {
        let obj = self.value
            .read()
            .map_err(|_| CacheLockError{})?;
        if obj.is_some() && !self.dirty.load(Ordering::Acquire) {
            Ok(obj.clone().unwrap())
        } else {
            drop(obj);    // unlock
            // First, acquire the write lock
            let mut obj = self.value
                .write()
                .map_err(|_| CacheLockError{})?;
            // With the write lock in hand, check if another thread
            // updated the object in the meantime.
            if self.dirty.load(Ordering::Relaxed) && obj.is_none() {
                let inner = (f)()?;
                let _ = obj.insert(inner.clone());
                Ok(inner)
            } else {
                // Another thread updated the object; use a recursive
                // call to fetch it.
                self.get(f)
            }
        }
    }

    pub fn set(&self, value: T) -> APIResult<()> {
        let mut obj = self.value
            .write()
            .map_err(|_| CacheLockError{})?;
        let _ = obj.insert(value);
        Ok(())
    }

    pub fn invalidate(&self) {
        self.dirty.store(true, Ordering::Release);
    }
}