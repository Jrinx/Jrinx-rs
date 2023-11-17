use alloc::vec::Vec;
use spin::{Mutex, MutexGuard};

pub struct MutexGroup<'a, T> {
    mutexes: Vec<&'a Mutex<T>>,
}

pub struct MutexGroupGuard<'a, T> {
    guards: Vec<MutexGuard<'a, T>>,
}

pub struct MutexGroupGuardIter<'a, 'b, T> {
    iter: core::slice::Iter<'a, MutexGuard<'b, T>>,
}

impl<'a, T> MutexGroup<'a, T> {
    pub fn new(mutexes: impl Iterator<Item = &'a Mutex<T>>) -> Self {
        Self {
            mutexes: mutexes.collect::<Vec<_>>(),
        }
    }

    pub fn lock(&self) -> MutexGroupGuard<'_, T> {
        loop {
            if let Some(guards) = self.try_lock_all() {
                return MutexGroupGuard { guards };
            }
            core::hint::spin_loop();
        }
    }

    fn try_lock_all(&self) -> Option<Vec<MutexGuard<'_, T>>> {
        let guards = self
            .mutexes
            .iter()
            .map(|mtx| mtx.try_lock())
            .collect::<Vec<_>>();
        if guards.iter().any(|guard| guard.is_none()) {
            None
        } else {
            guards.into_iter().collect::<Option<Vec<_>>>()
        }
    }
}

impl<'a, 'b, T> MutexGroupGuard<'b, T> {
    pub fn iter(&'b self) -> MutexGroupGuardIter<'a, 'b, T> {
        MutexGroupGuardIter {
            iter: self.guards.iter(),
        }
    }
}

impl<'a, T> IntoIterator for MutexGroupGuard<'a, T> {
    type IntoIter = <Vec<MutexGuard<'a, T>> as IntoIterator>::IntoIter;

    type Item = MutexGuard<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.guards.into_iter()
    }
}

impl<'a, 'b, T> Iterator for MutexGroupGuardIter<'a, 'b, T> {
    type Item = &'a MutexGuard<'b, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
