use crate::mem::stack;
use crate::multitasking::preemptive::ThreadID;
use x86_64::VirtAddr;
use x86_64::structures::paging::{FrameAllocator, Mapper, Size4KiB};

use super::SCHEDULER;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Thread {
    id: ThreadID,
    pub(super) quantum: u64,
    pub(super) done: bool,
    //    pub(super) stackBounds: StackBounds,
    pub(super) stackPointer: VirtAddr,
    pub(super) instructionPointer: VirtAddr,
    pub(super) function: extern "C" fn(),
    //  registers: [u64; 23], // rax, rbx, rcx, rdx, rbp, rsi, rdi, r8, r9, r10, r11, r12, r13, r14, r15, ax, gs, fs, es, ds, ss, cs, cr3, rflags
}

impl Thread {
    pub fn new<M, A>(func: extern "C" fn(), mapper: &mut M, frameAllocator: &mut A) -> Thread
    where
        M: Mapper<Size4KiB>,
        A: FrameAllocator<Size4KiB>,
    {
        //+ Send + 'static
        let sb = stack::allocStack(1, mapper, frameAllocator).ok();
        if sb.is_none() {
            panic!("Failed to allocate stack");
        }

        Thread {
            id: ThreadID::new(),
            quantum: 20,
            done: false,
            //     stackBounds: sb.unwrap(),
            stackPointer: sb.unwrap().start,
            instructionPointer: VirtAddr::new(func as *const () as u64),
            function: func,
            //      registers: [0; 23],
        }
    }

    pub unsafe fn spawn(self) {
        SCHEDULER.lock().schedule(self);
    }
}

// #[derive(Debug, Clone, Copy)]
// pub struct Thread {
//     id: ThreadID,
//     stackPointer: Option<VirtAddr>,
//     stackBounds: Option<StackBounds>,
// }

// impl Thread {
//     pub fn new(
//         id: ThreadID,
//         stackPointer: Option<VirtAddr>,
//         stackBounds: Option<StackBounds>,
//     ) -> Self {
//         Self {
//             id,
//             stackPointer,
//             stackBounds,
//         }
//     }

//     pub fn getID(&self) -> ThreadID {
//         self.id
//     }

//     pub fn getStackPointer(&self) -> Option<VirtAddr> {
//         self.stackPointer
//     }

//     pub fn getStackBounds(&self) -> Option<StackBounds> {
//         self.stackBounds
//     }
// }
