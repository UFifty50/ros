use core::sync::atomic::AtomicPtr;


pub struct AtomicSingleton<T>
where T: Default {
    ptr: AtomicPtr<T>,
}

impl<T> AtomicSingleton<T>
where T: Default {
    // TODO: implement
}
