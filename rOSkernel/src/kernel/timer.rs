use alloc::collections::BinaryHeap;
use crate::multitasking::preemptive::ThreadID;


#[derive(Debug, PartialEq, Eq)]
struct Timer {
    deadline: u64,
    payload: TimerPayload,
}

#[derive(Debug, PartialEq, Eq)]
enum TimerPayload {
    WakeThread(ThreadID),
    // DeferSignal { threadID: ThreadID, signal: Signal },
    DeferImportant(ThreadID),
}

impl Ord for Timer {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        // Timers are ordered by earliest deadline
        other.deadline.cmp(&self.deadline)
    }
}

impl PartialOrd for Timer {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}


#[derive(Debug)]
pub struct TimerQueue {
    heap: BinaryHeap<Timer>,
}

impl TimerQueue {
    pub fn new() -> Self {
        TimerQueue {
            heap: BinaryHeap::new(),
        }
    }

    pub fn addTimer(&mut self, deadline: u64, payload: TimerPayload) {
        let timer = Timer { deadline, payload };
        self.heap.push(timer);
    }

    pub fn popExpiredTimers(&mut self, now: u64) {
        while let Some(timer) = self.heap.peek() {
            if timer.deadline > now {
                break;
            }

            let expired_timer = self.heap.pop().unwrap();
            match expired_timer.payload {
                TimerPayload::WakeThread(_thread_id) => {
                    // Wake the thread with thread_id
                    // (Implementation depends on the rest of the kernel)
                }
                TimerPayload::DeferImportant(_thread_id) => {
                    // Handle defer important for thread_id
                    // (Implementation depends on the rest of the kernel)
                }
            }
        }
    }
}
