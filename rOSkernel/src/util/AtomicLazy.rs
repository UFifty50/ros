use core::sync::atomic::AtomicPtr;

pub struct AtomicLazy<T> {
    inner: AtomicPtr<T>,
}

impl<T> AtomicLazy<T>
where
    T: Default,
{
    // TODO: implement
}
