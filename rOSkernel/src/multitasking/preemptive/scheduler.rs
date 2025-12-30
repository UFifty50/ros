use super::thread::{GPRegisters, InterruptFrame};
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::PhysFrame;
use crate::kernel::kernelContext;
use crate::multitasking::preemptive::thread::Process;
use crate::multitasking::preemptive::{ProcessID, ThreadID};
use crate::util::wrappers::{xgetbv0, xsetbv0, CPUID, XFeatures, get_fpu_mechanism, FpuSaveMechanism};
use alloc::alloc::{alloc, dealloc, Layout};
use alloc::sync::Arc;

/// Data extracted from a thread for context switching
#[derive(Clone, Copy)]
struct ThreadContext {
    cr3: PhysFrame,
    gpRegisters: GPRegisters,
    iFrame: InterruptFrame,
    xAreaPtr: Option<*mut u8>,
    xFeatures: XFeatures,
}

pub struct Scheduler {
    processes: BTreeMap<ProcessID, Arc<Process>>,
    run_queue: Vec<(ProcessID, ThreadID)>,
    currentThreadIdx: Option<usize>,
}


impl Scheduler {
    pub const fn new() -> Self {
        Self {
            processes: BTreeMap::new(),
            run_queue: Vec::new(),
            currentThreadIdx: None,
        }
    }

    pub fn schedule(&mut self, pid: ProcessID, tid: ThreadID) {
        self.run_queue.push((pid, tid));
    }

    pub fn register_process(&mut self, process: Arc<Process>) {
        self.processes.insert(process.pid(), process);
    }

    pub fn get_process(&self, pid: ProcessID) -> Option<Arc<Process>> {
        self.processes.get(&pid).cloned()
    }

    /// Returns the PID of the currently running process, if any.
    pub fn current_pid(&self) -> Option<ProcessID> {
        self.currentThreadIdx.map(|idx| self.run_queue[idx].0)
    }

    pub fn switchTask(
        &mut self,
        saved_regs: *mut GPRegisters,
        frame: *const InterruptFrame,
    ) -> *mut GPRegisters {
        let frame = unsafe { &*frame };

        // Save current thread state
        if let Some(currentIdx) = self.currentThreadIdx {
            let (pid, tid) = self.run_queue[currentIdx];
            
            if let Some(process) = self.processes.get(&pid) {
                let hasTimeLeft = process.with_thread_mut(&tid, |thread| {
                    let Some(current) = thread else { return false };
                    if current.initialised && current.quantum > 0 {
                        current.quantum -= 1;
                        return true;
                    }

                    current.quantum = current.maxQuantum;
                    current.gpRegisters = unsafe { *saved_regs };
                    current.iFrame = *frame;

                    // Save extended state
                    if let Some(ptr) = current.xAreaPtr {
                        match get_fpu_mechanism() {
                            FpuSaveMechanism::FXSave => unsafe {
                                core::arch::x86_64::_fxsave64(ptr);
                            },
                            FpuSaveMechanism::XSave => unsafe {
                                current.xFeatures = XFeatures(xgetbv0());
                                core::arch::x86_64::_xsave64(ptr, u64::MAX);
                            },
                            FpuSaveMechanism::None => {},
                        }
                    }

                    false
                });
                
                if hasTimeLeft {
                    return saved_regs;
                }
            }
        }

        let nextIdx = match self.currentThreadIdx {
            Some(currentIdx) => (currentIdx + 1) % self.run_queue.len(),
            None => 0,
        };
        self.currentThreadIdx = Some(nextIdx);

        let (next_pid, next_tid) = self.run_queue[nextIdx];
        let process = self.processes.get(&next_pid).expect("Process in run_queue not found");
        
        // Extract context and handle XSAVE resize atomically within the closure
        let ctx = process.with_thread_mut(&next_tid, |thread| {
            let next = thread.expect("Thread in run_queue not found");
            
            if !next.initialised {
                next.initialised = true;
            }
            
            // Handle XSAVE resize if needed (bits can be enabled but not disabled, so area will only grow)
            if get_fpu_mechanism() == FpuSaveMechanism::XSave {
                // Check if resize is needed (read-only check)
                let newSize = Self::getXAreaSize(next.xFeatures);
                
                if newSize != next.xAreaSize {
                    // Perform resize - allocate new, copy, deallocate old, update thread
                    let new_ptr = unsafe {
                        let new_layout = Layout::from_size_align(newSize as usize, next.xAreaAlign as usize).unwrap();
                        let new_ptr = alloc(new_layout);
                        if new_ptr.is_null() {
                            panic!("Failed to reallocate XSAVE area for next thread");
                        }
                        core::ptr::write_bytes(new_ptr, 0, newSize as usize);
                        
                        if let Some(old_ptr) = next.xAreaPtr {
                            core::ptr::copy_nonoverlapping(old_ptr, new_ptr, next.xAreaSize as usize);
                            let old_layout = Layout::from_size_align(next.xAreaSize as usize, next.xAreaAlign as usize).unwrap();
                            dealloc(old_ptr as *mut u8, old_layout);
                        } else {
                            // Initialize legacy region defaults for new area
                            *(new_ptr as *mut u16) = 0x037F;
                            *(new_ptr.add(24) as *mut u32) = 0x1F80;
                        }
                        new_ptr
                    };
                    
                    next.xAreaPtr = Some(new_ptr);
                    next.xAreaSize = newSize;
                }
            }
            
            ThreadContext {
                cr3: next.cr3,
                gpRegisters: next.gpRegisters,
                iFrame: next.iFrame,
                xAreaPtr: next.xAreaPtr,
                xFeatures: next.xFeatures,
            }
        });
        
        // Switch address space if needed
        let (currentCR3, _) = Cr3::read();
        if currentCR3 != ctx.cr3 {
            unsafe { Cr3::write(ctx.cr3, Cr3::read().1); }
        }

        // Build return frame on thread's stack
        let frame_ptr = (ctx.iFrame.rsp - size_of::<InterruptFrame>() as u64) as *mut InterruptFrame;
        let regs_ptr = (frame_ptr as u64 - size_of::<GPRegisters>() as u64) as *mut GPRegisters;

        unsafe {
            *regs_ptr = ctx.gpRegisters;
            (*frame_ptr).ss = ctx.iFrame.ss;
            (*frame_ptr).rsp = ctx.iFrame.rsp;
            (*frame_ptr).rflags = ctx.iFrame.rflags;
            (*frame_ptr).cs = ctx.iFrame.cs;
            (*frame_ptr).rip = ctx.iFrame.rip;
        }

        // Restore extended state
        if let Some(ptr) = ctx.xAreaPtr {
            match get_fpu_mechanism() {
                FpuSaveMechanism::FXSave => unsafe {
                    core::arch::x86_64::_fxrstor64(ptr);
                },
                FpuSaveMechanism::XSave => unsafe {
                    // Set XCR0 before restore
                    xsetbv0(ctx.xFeatures.to_u64());
                    core::arch::x86_64::_xrstor64(ptr, ctx.xFeatures.to_u64());
                },
                FpuSaveMechanism::None => {},
            }
        }

        regs_ptr
    }
    
    fn getXAreaSize(xFeatures: XFeatures) -> u32 {
        unsafe {
            // Validate features are supported
            let (eax, _, _, edx) = CPUID(0xD, 0);
            let supported = (edx as u64) << 32 | eax as u64;
            if (xFeatures.to_u64() & !supported) != 0 {
                panic!("Thread trying to enable unsupported xFeatures: {:x}", xFeatures.to_u64());
            }

            // Calculate size needed for these features
            // Note: We need to temporarily set XCR0 to query the size, then restore it
            let current_xcr0 = xgetbv0();
            xsetbv0(xFeatures.to_u64());
            let size_needed = CPUID(0xD, 0).1;
            xsetbv0(current_xcr0);

            size_needed
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn timer_interrupt_trampoline(
    savedRegs: *mut GPRegisters,
    frame: *const InterruptFrame,
) -> *mut GPRegisters {
    use crate::multitasking::preemptive::SCHEDULER;
    let mut scheduler = SCHEDULER.lock();

    let res: *mut GPRegisters;
    if scheduler.run_queue.is_empty() {
        res = savedRegs;
    } else {
        res = scheduler.switchTask(savedRegs, frame);
    }

    kernelContext()
        .apic
        .get()
        .expect("APIC not initialized.")
        .notifyEOI();
    
    res
}