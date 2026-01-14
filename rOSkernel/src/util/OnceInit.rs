use core::ops::{Deref, DerefMut};
use spin::Mutex;

/// A thread-safe container that can be initialized once.
/// After initialization, you can access the inner value mutably.
pub struct OnceInit<T> {
    inner: Mutex<Option<T>>,
}

impl<T> OnceInit<T> {
    /// Create a new, uninitialized instance.
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }

    /// Gets a mutable guard to the inner value. If the value is not
    /// yet initialized, it will be created by calling the provided closure.
    ///
    /// This allows you to forward any arguments to `T::new` by writing:
    ///
    /// ```rust
    /// use rOSkernel::util::OnceInit::OnceInit;
    /// let once = OnceInit::new();
    /// let guard = once.get_or_init(|| String::new());
    /// ```
    ///
    /// The returned guard implements `DerefMut` so you can modify the inner value.
    pub fn get_or_init<F>(&self, f: F) -> OnceMutGuard<'_, T>
    where
        F: FnOnce() -> T,
    {
        let mut guard = self.inner.lock();
        if guard.is_none() {
            *guard = Some(f());
        }
        OnceMutGuard { inner: guard }
    }

    /// Get mutable access to the contained value.
    /// Returns `None` if not yet initialized.
    pub fn get_mut(&self) -> Option<OnceMutGuard<'_, T>> {
        let guard = self.inner.lock();
        // We only create a mutable guard if the value is actually initialized.
        if guard.is_some() {
            Some(OnceMutGuard { inner: guard })
        } else {
            None
        }
    }
}

impl<T: Copy> OnceInit<T> {
    pub fn get_copy(&self) -> Option<T> {
        let guard = self.inner.lock();
        guard.as_ref().copied()
    }
}

impl<T: Clone> OnceInit<T> {
    pub fn get_clone(&self) -> Option<T> {
        let guard = self.inner.lock();
        guard.as_ref().cloned()
    }
}

/// A mutable guard that provides mutable access to the inner T.
/// Dropping the guard unlocks the inner mutex.
pub struct OnceMutGuard<'a, T> {
    inner: spin::MutexGuard<'a, Option<T>>,
}

impl<'a, T> Deref for OnceMutGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // Unwrap is safe here because we only construct this guard if the Option is Some.
        self.inner.as_ref().unwrap()
    }
}

impl<'a, T> DerefMut for OnceMutGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.inner.as_mut().unwrap()
    }
}
