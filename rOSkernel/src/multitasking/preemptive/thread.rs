use crate::mem::stack::{self, StackBounds};
use crate::util::wrappers::{XFeatures, xgetbv0, xsetbv0, get_fpu_mechanism, FpuSaveMechanism};
use crate::multitasking::preemptive::{ProcessID, ThreadID};
use alloc::collections::BTreeMap;
use x86_64::VirtAddr;
use x86_64::instructions::interrupts;
use x86_64::structures::paging::{PhysFrame, OffsetPageTable, PageTable};
use crate::kernel::kernelContext;
use super::{SCHEDULER, Parent, current_pid};
use alloc::alloc::{alloc, dealloc, Layout};
use crate::mem::memory::{newAddressSpace, PHYSICAL_MEMORY_OFFSET};
use alloc::sync::Arc;
use spin::Mutex;
use cpuid::CPUID;

pub type ProcessRef = Arc<Process>;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(C)]
pub struct GPRegisters {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct InterruptFrame {
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct SegmentRegisters {
    pub cs: u16,
    pub ss: u16,
    pub fs: u64,
    pub gs: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadStatus {
    Spawned,
    Sleeping,
    SleepingNoDisturb,
    Waking, // foresight for multicore, prevents a task from being woken up twice
    Dead,
}

#[derive(Debug)]
#[repr(C)]
pub struct Thread {
    id: ThreadID,
    pub parentPID: ProcessID,
    pub(super) maxQuantum: u64,
    pub(super) quantum: u64,
    pub(super) done: bool,
    pub status: ThreadStatus,
    pub initialised: bool,
    pub stackBounds: StackBounds,

    // registers
    pub cr3: PhysFrame,
    pub gpRegisters: GPRegisters,
    pub iFrame: InterruptFrame,

    // xsave area
    pub xAreaPtr: Option<*mut u8>,
    pub xAreaSize: u32,
    pub xAreaAlign: u32,
    pub xFeatures: XFeatures,

    pub(super) function: extern "C" fn(),
}

#[derive(Debug)]
pub struct Process {
    pid: ProcessID,
    parentPID: Option<ProcessID>,
    pageTable: PhysFrame,
    threads: Mutex<BTreeMap<ThreadID, Thread>>,
    // TODO: file descriptors, etc.
}

impl Drop for Process {
    fn drop(&mut self) {
        // threads are dropped automatically when the BTreeMap is dropped
        self.pid.free();
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        // Deallocate xsave/fxsave area with correct alignment
        if let Some(ptr) = self.xAreaPtr {
            if self.xAreaSize > 0 {
                let layout = Layout::from_size_align(self.xAreaSize as usize, self.xAreaAlign as usize)
                    .expect("Invalid xArea layout");
                unsafe {
                    dealloc(ptr, layout);
                }
            }
            self.xAreaPtr = None;
        }
        
        // Deallocate stack pages
        // Note: This requires unmapping the pages from the page table
        // For now we track the bounds; full deallocation requires mapper access
        // TODO: Implement stack::deallocStack when page table access is available
    }
}

impl Process {
    pub fn create(parent: Parent) -> ProcessRef {
        let parent_pid = match parent {
            Parent::Inherit => current_pid(),
            Parent::Independent => None,
            Parent::Explicit(pid) => Some(pid),
        };

        let mut frameAllocatorGuard = kernelContext().frameAllocator.get().unwrap().lock();
        let phys_offset = VirtAddr::new(PHYSICAL_MEMORY_OFFSET.get_copy().unwrap());
        
        // Create new address space
        let new_cr3 = newAddressSpace(&mut *frameAllocatorGuard, phys_offset)
            .expect("Failed to allocate new address space");

        let pid = ProcessID::new();
        let process = Process {
            pid,
            parentPID: parent_pid,
            pageTable: new_cr3,
            threads: Mutex::new(BTreeMap::new()),
        };
        
        let process_arc = Arc::new(process);
        
        interrupts::without_interrupts(|| {
            let mut guard = SCHEDULER.lock();
            guard.register_process(process_arc.clone());
        });
        
        process_arc
    }

    pub fn pid(&self) -> ProcessID {
        self.pid
    }

    pub fn with_thread_mut<F, R>(&self, tid: &ThreadID, f: F) -> R
    where
        F: FnOnce(Option<&mut Thread>) -> R,
    {
        interrupts::without_interrupts(|| {
            let mut threads_lock = self.threads.lock();
            let thread = threads_lock.get_mut(tid);
            f(thread)
        })
    }

    pub fn create_thread(&self, func: extern "C" fn(), maxQuantum: u64) -> ThreadID {
        let phys_offset = VirtAddr::new(PHYSICAL_MEMORY_OFFSET.get_copy().unwrap());

        let l4_table_ptr = (phys_offset + self.pageTable.start_address().as_u64()).as_mut_ptr::<PageTable>();
        let l4_table = unsafe { &mut *l4_table_ptr };
        let mut mapper = unsafe { OffsetPageTable::new(l4_table, phys_offset) };
        
        let mut frameAllocatorGuard = kernelContext().frameAllocator.get().unwrap().lock();
        
        // Allocate 4 pages (16KB) for stack
        let stackPageCount = 4u64;
        let sb = stack::allocStack(stackPageCount, &mut mapper, &mut *frameAllocatorGuard).ok();
        if sb.is_none() {
            panic!("Failed to allocate stack");
        }
        let stackBounds = sb.unwrap();

        let (cs, ss): (u16, u16);
        unsafe {
            core::arch::asm!("mov {0:x}, cs", out(reg) cs, options(nomem, nostack, preserves_flags));
            core::arch::asm!("mov {0:x}, ss", out(reg) ss, options(nomem, nostack, preserves_flags));
        }

        let xFeatures = if get_fpu_mechanism() == FpuSaveMechanism::XSave {
            XFeatures::current()
        } else {
            XFeatures::new(0)
        };

        let (fx_ptr, fx_size, fx_align) = match get_fpu_mechanism() {
            FpuSaveMechanism::FXSave => unsafe {
                let size = 512;
                let align = 16usize;
                let layout = Layout::from_size_align(size, align).unwrap();
                let ptr = alloc(layout);
                if ptr.is_null() {
                    panic!("Failed to allocate FX save area");
                }
                core::ptr::write_bytes(ptr, 0, size);
                // Set FCW to 0x037F
                *(ptr as *mut u16) = 0x037F;
                // Set MXCSR to 0x1F80 (offset 24)
                *(ptr.add(24) as *mut u32) = 0x1F80;
                (Some(ptr), size as u32, align as u32)
            },
            FpuSaveMechanism::XSave => unsafe {
                // XSAVE logic (currently disabled via initXFeatures, but kept here for completeness)
                let size_ebx = x86_64::instructions::interrupts::without_interrupts(|| {
                    let current_xcr0 = xgetbv0();
                    xsetbv0(xFeatures.to_u64());
                    let size = CPUID::xsaveInfo().unwrap().currentMaxSaveArea;
                    xsetbv0(current_xcr0);
                    size
                });
                
                let align = 64usize;
                let layout = Layout::from_size_align(size_ebx as usize, align).unwrap();
                let ptr = alloc(layout);
                if ptr.is_null() {
                    panic!("Failed to allocate XSAVE area");
                }
                core::ptr::write_bytes(ptr, 0, size_ebx as usize);
                *(ptr as *mut u16) = 0x037F;
                *(ptr.add(24) as *mut u32) = 0x1F80;
                (Some(ptr), size_ebx, align as u32)
            },
            FpuSaveMechanism::None => (None, 0, 1),
        };

        let newThreadID = ThreadID::new();
        let newThread = Thread {
            id: newThreadID,
            parentPID: self.pid,
            maxQuantum,
            quantum: maxQuantum,
            done: false,
            initialised: false,
            status: ThreadStatus::Spawned,
            cr3: self.pageTable,
            gpRegisters: GPRegisters::default(),
            iFrame: InterruptFrame {
                rip: func as *const () as u64,
                cs: cs as u64,
                rflags: 0x202,
                rsp: stackBounds.end.as_u64(),
                ss: ss as u64,
            },
            xAreaPtr: fx_ptr,
            xAreaSize: fx_size,
            xAreaAlign: fx_align,
            xFeatures,
            function: func,
            stackBounds,
        };

        interrupts::without_interrupts(|| {
            let mut threads_lock = self.threads.lock();
            threads_lock.insert(newThreadID, newThread);
        });
        
        newThreadID
    }

    pub fn add_thread_xfeatures(&self, tid: &ThreadID, features: XFeatures) -> Result<(), ()> {
        interrupts::without_interrupts(|| {
            let mut threads_lock = self.threads.lock();
            if let Some(thread) = threads_lock.get_mut(tid) {
                thread.xFeatures.0 |= features.to_u64();
                Ok(())
            } else {
                Err(())
            }
        })
    }

    pub fn start_thread(&self, tid: ThreadID) -> Option<()> {
        interrupts::without_interrupts(|| {
            if !self.threads.lock().contains_key(&tid) {
                return None;
            }

            let mut guard = SCHEDULER.lock();
            guard.schedule(self.pid, tid)
        })
    }

}

unsafe impl Send for Thread {}
