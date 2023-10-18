use crate::mem::stack::{self, StackBounds};
use crate::multitasking::preemptive::ThreadID;
use x86_64::structures::paging::{FrameAllocator, Mapper, Size4KiB};

use super::SCHEDULER;

#[derive(Debug, Clone, Copy)]
pub struct Thread {
    id: ThreadID,
    pub(super) quantum: u64,
    pub(super) done: bool,
    pub(super) stackPointer: Option<u64>,
    stackBounds: Option<StackBounds>,
    pub(super) function: fn(),
}

impl Thread {
    pub fn new<M, A>(func: fn(), mapper: &mut M, frameAllocator: &mut A) -> Thread
    where
        M: Mapper<Size4KiB>,
        A: FrameAllocator<Size4KiB>,
    {
        //+ Send + 'static
        Thread {
            id: ThreadID::new(),
            quantum: 5,
            done: false,
            stackPointer: None,
            stackBounds: stack::allocStack(1, mapper, frameAllocator).ok(),
            function: func,
        }
    }

    pub unsafe fn spawn(&mut self) {
        SCHEDULER.schedule(*self);
    }

    pub fn getStackPointer(&self) -> Option<u64> {
        if self.stackPointer.is_some() {
            Some(self.stackPointer.unwrap())
        } else {
            None
        }
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
