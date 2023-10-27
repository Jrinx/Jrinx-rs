use core::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicU8, Ordering},
};

use jrinx_error::{InternalError, Result};

const UNINIT: u8 = 0;
const INITIALIZING: u8 = 1;
const INITED: u8 = 2;

pub struct OnceLock<T> {
    state: AtomicU8,
    value: UnsafeCell<MaybeUninit<T>>,
}

impl<T> OnceLock<T> {
    pub const fn new() -> Self {
        OnceLock {
            state: AtomicU8::new(UNINIT),
            value: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    pub fn init(&self, value: T) -> Result<()> {
        if let Ok(UNINIT) =
            self.state
                .compare_exchange(UNINIT, INITIALIZING, Ordering::Acquire, Ordering::Relaxed)
        {
            unsafe {
                self.value.get().write(MaybeUninit::new(value));
            }
            self.state.store(INITED, Ordering::Release);
            Ok(())
        } else {
            Err(InternalError::RepeatInitialization)
        }
    }

    pub fn get(&self) -> Option<&T> {
        if self.state.load(Ordering::Acquire) == INITED {
            Some(unsafe { (*self.value.get()).assume_init_ref() })
        } else {
            None
        }
    }
}

unsafe impl<T: Send> Send for OnceLock<T> {}
unsafe impl<T: Send + Sync> Sync for OnceLock<T> {}
