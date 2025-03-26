use self::scheduler::Scheduler;
use spin::Mutex;

pub mod scheduler;
pub mod switchThread;
pub mod thread;

pub static SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct ThreadID(u64);

impl ThreadID {
    pub fn as_u64(&self) -> u64 {
        self.0
    }

    pub fn new() -> Self {
        use core::sync::atomic::{AtomicU64, Ordering};
        static NEXT_THREAD_ID: AtomicU64 = AtomicU64::new(1);
        ThreadID(NEXT_THREAD_ID.fetch_add(1, Ordering::Relaxed))
    }
}
