use super::{thread::Thread, SCHEDULER};
use alloc::vec::Vec;

pub struct Scheduler {
    threads: Vec<Thread>,
    currentThreadIdx: usize,
    previousThreadIdx: usize,
}

impl Scheduler {
    pub const fn new() -> Self {
        Self {
            threads: Vec::new(),
            currentThreadIdx: 0,
            previousThreadIdx: 0,
        }
    }

    pub fn schedule(&mut self, thread: Thread) {
        self.threads.push(thread);
        //  (self.threads[self.threads.len() - 1].function)(); //TODO: When do i run you????
    }

    pub fn switchTask(&mut self) {
        if self.threads.len() == 0 {
            return;
        }

        if self.threads[self.currentThreadIdx].quantum == 0 {
            self.threads[self.currentThreadIdx].quantum = 5;
            self.previousThreadIdx = self.currentThreadIdx;
            self.currentThreadIdx = (self.currentThreadIdx + 1) % self.threads.len();

            if let Some(newStackPointer) = self.threads[self.currentThreadIdx].getStackPointer() {
                unsafe {
                    asmSwitchThread(newStackPointer);
                };
            } else {
                if self.threads[self.currentThreadIdx].done {
                    self.threads.remove(self.currentThreadIdx);
                } else {
                    (self.threads[self.previousThreadIdx].function)();
                }
            }
        } else {
            self.threads[self.currentThreadIdx].quantum -= 1;
        }
    }
}

// declare external assembly function
extern "C" {
    pub fn asmSwitchThread(newStackPointer: u64);
}

// declare function called from external assembly
#[no_mangle]
pub unsafe extern "C" fn addPausedThread(oldStackPointer: u64) {
    SCHEDULER.threads[SCHEDULER.previousThreadIdx].stackPointer = Some(oldStackPointer);
}
