use self::scheduler::Scheduler;
use spin::Mutex;
use alloc::collections::BTreeSet;

pub mod scheduler;
pub mod switchThread;
pub mod thread;

pub static SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());

/// Specifies the parent process relationship when creating a new process.
/// 
/// - `Inherit` (default): Automatically inherits the currently running process as parent
/// - `Independent`: The process has no parent (like PID 1 / init)
/// - `Explicit(ProcessID)`: Explicitly set a specific process as the parent
#[derive(Debug, Clone, Copy)]
pub enum Parent {
    /// Inherit the currently running process as parent (default behavior)
    Inherit,
    /// No parent - independent process (equivalent to passing None explicitly)
    Independent,
    /// Explicitly specify a parent process
    Explicit(ProcessID),
}

impl Default for Parent {
    fn default() -> Self {
        Parent::Inherit
    }
}

impl From<Option<ProcessID>> for Parent {
    fn from(opt: Option<ProcessID>) -> Self {
        match opt {
            Some(pid) => Parent::Explicit(pid),
            None => Parent::Independent,
        }
    }
}

/// Returns the PID of the currently running process, if any.
/// Returns None if no process is currently scheduled (e.g., during early boot).
pub fn current_pid() -> Option<ProcessID> {
    x86_64::instructions::interrupts::without_interrupts(|| {
        SCHEDULER.lock().current_pid()
    })
}
pub static PROCESS_ID_ALLOCATOR: Mutex<IDAllocator> = Mutex::new(IDAllocator::new());

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


pub struct IDAllocator {
    next_id: u64,
    freed_ids: BTreeSet<u64>,
}

impl IDAllocator {
    pub const fn new() -> Self {
        Self {
            next_id: 1,
            freed_ids: BTreeSet::new(),
        }
    }

    pub fn allocate(&mut self) -> u64 {
        if let Some(&id) = self.freed_ids.iter().next() {
            // We need to clone the id because we can't move out of a reference
            let id = id.clone();
            self.freed_ids.remove(&id);
            id
        } else {
            let id = self.next_id;
            self.next_id += 1;
            id
        }
    }

    pub fn deallocate(&mut self, id: u64) {
        self.freed_ids.insert(id);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct ProcessID(u64);

impl ProcessID {
   pub fn as_u64(&self) -> u64 {
        self.0
    }

    pub fn new() -> Self {
        let mut allocator = PROCESS_ID_ALLOCATOR.lock();
        ProcessID(allocator.allocate())
    }

    pub fn free(self) {
        let mut allocator = PROCESS_ID_ALLOCATOR.lock();
        allocator.deallocate(self.0);
    }
}
