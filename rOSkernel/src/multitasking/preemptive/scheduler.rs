use super::{thread::Thread, SCHEDULER};
use alloc::vec::Vec;
use x86_64::VirtAddr;

pub struct Scheduler {
    threads: Vec<Thread>,
    currentThreadIdx: Option<usize>,
    previousThreadIdx: Option<usize>,
}

impl Scheduler {
    pub const fn new() -> Self {
        Self {
            threads: Vec::new(),
            currentThreadIdx: None,
            previousThreadIdx: None,
        }
    }

    pub fn schedule(&mut self, thread: Thread) {
        self.threads.push(thread);
    }

    pub fn switchTask(&mut self) {
        if self.threads.len() == 0 {
            log::info!("No threads to switch to");
            return;
        }

        // round robin
        if self.currentThreadIdx.is_none() {
            self.currentThreadIdx = Some(0);
            self.previousThreadIdx = Some(0);
        } else {
            self.previousThreadIdx = self.currentThreadIdx;
            self.currentThreadIdx = Some((self.currentThreadIdx.unwrap() + 1) % self.threads.len());
        }

        self.threads[self.currentThreadIdx.unwrap()].quantum -= 1;
        
        if self.threads[self.currentThreadIdx.unwrap()].quantum > 0 {
            return;
        }

        self.threads[self.currentThreadIdx.unwrap()].quantum = 20;
        self.previousThreadIdx = self.currentThreadIdx;
        self.currentThreadIdx = Some((self.currentThreadIdx.unwrap() + 1) % self.threads.len());

        unsafe {
            log::info!(
                "Switching to thread: {} with quantum: {} and stack pointer: 0x{:x}",
                self.currentThreadIdx.unwrap(),
                self.threads[self.currentThreadIdx.unwrap()].quantum,
                self.threads[self.currentThreadIdx.unwrap()]
                    .stackPointer
                    .as_u64()
            );

            asmSwitchThread(
                self.threads[self.currentThreadIdx.unwrap()]
                    .stackPointer
                    .as_u64(),
                self.threads[self.currentThreadIdx.unwrap()]
                    .instructionPointer
                    .as_u64(),
            );
        }
    }
}

// declare external assembly function

unsafe extern "C" {
    pub fn asmSwitchThread(newStackPtr: u64, newIP: u64);

}

// declare function called from external assembly
#[unsafe(no_mangle)]
pub unsafe extern "C" fn addPausedThread(oldStackPointer: u64, oldInstructionPointer: u64) {
    log::info!("oldStackPointer: 0x{:x}", oldStackPointer);
    let previousThreadIdx = SCHEDULER.lock().previousThreadIdx.unwrap();
    SCHEDULER.lock().threads[previousThreadIdx].stackPointer = VirtAddr::new(oldStackPointer);
    SCHEDULER.lock().threads[previousThreadIdx].instructionPointer = VirtAddr::new(oldInstructionPointer);
}
