use super::thread::{GPRegisters, InterruptFrame, ThreadStatus, ProcessRef};
use alloc::collections::vec_deque::VecDeque;
use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::PhysFrame;
use crate::kernel::kernelContext;
use crate::multitasking::preemptive::{ProcessID, ThreadID};
use crate::util::wrappers::{xgetbv0, xsetbv0, CPUID, XFeatures, get_fpu_mechanism, FpuSaveMechanism};
use alloc::alloc::{alloc, dealloc, Layout};

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
    processes: BTreeMap<ProcessID, ProcessRef>,
    current: Option<(ProcessID, ThreadID)>,
    ready: VecDeque<(ProcessID, ThreadID)>,
    blocked: BTreeSet<(ProcessID, ThreadID)>,
    idle: Option<(ProcessID, ThreadID)>,
}


impl Scheduler {
    pub const fn new() -> Self {
        Self {
            processes: BTreeMap::new(),
            current: None,
            ready: VecDeque::new(),
            blocked: BTreeSet::new(),
            idle: None,
        }
    }

    pub fn set_idle(&mut self, pid: ProcessID, tid: ThreadID) {
        self.idle = Some((pid, tid));
    }

    pub fn schedule(&mut self, pid: ProcessID, tid: ThreadID) -> Option<()> {
        let process = self.processes.get(&pid)?;
        
        let status = process.with_thread_mut(&tid, |thread| {
            thread.map(|t| t.status)
        })?;
        
        if status == ThreadStatus::Dead {
            return None;
        }
        
        if self.ready.contains(&(pid, tid)) {
            return None;
        }
        
        self.blocked.remove(&(pid, tid));
        
        self.ready.push_back((pid, tid));
        Some(())
    }

    pub fn register_process(&mut self, process: ProcessRef) {
        self.processes.insert(process.pid(), process);
    }

    pub fn unregister_process(&mut self, pid: ProcessID) -> Option<ProcessRef> {
        self.ready.retain(|(p, _)| *p != pid);
        self.blocked.retain(|(p, _)| *p != pid);
        if self.current.map(|(p, _)| p) == Some(pid) {
            self.current = None;
        }
        self.processes.remove(&pid)
    }

    pub fn get_process(&self, pid: ProcessID) -> Option<ProcessRef> {
        self.processes.get(&pid).cloned()
    }

    pub fn current(&self) -> Option<(ProcessID, ThreadID)> {
        self.current
    }

    pub fn current_pid(&self) -> Option<ProcessID> {
        self.current.map(|(pid, _)| pid)
    }

    pub fn sleep(&mut self, pid: ProcessID, tid: ThreadID) -> Option<()> {
        let process = self.processes.get(&pid)?;
        
        process.with_thread_mut(&tid, |thread| {
            let t = thread?;
            t.status = ThreadStatus::Sleeping;
            Some(())
        })?;
        
        self.ready.retain(|&x| x != (pid, tid));
        self.blocked.insert((pid, tid));
        
        Some(())
    }

    pub fn sleep_no_disturb(&mut self, pid: ProcessID, tid: ThreadID) -> Option<()> {
        let process = self.processes.get(&pid)?;
        
        process.with_thread_mut(&tid, |thread| {
            let t = thread?;
            t.status = ThreadStatus::SleepingNoDisturb;
            Some(())
        })?;
        
        self.ready.retain(|&x| x != (pid, tid));
        self.blocked.insert((pid, tid));
        
        Some(())
    }

    pub fn wake(&mut self, pid: ProcessID, tid: ThreadID) -> Option<()> {
        let process = self.processes.get(&pid)?;
        
        let should_enqueue = process.with_thread_mut(&tid, |thread| {
            let t = thread?;
            
            match t.status {
                ThreadStatus::Sleeping => {
                    t.status = ThreadStatus::Waking;
                    Some(true)
                }
                ThreadStatus::Spawned => {
                    Some(false)
                }
                _ => None,
            }
        })?;
        
        if should_enqueue {
            self.blocked.remove(&(pid, tid));
            self.ready.push_back((pid, tid));
        }
        
        Some(())
    }

    pub fn wake_force(&mut self, pid: ProcessID, tid: ThreadID) -> Option<()> {
        let process = self.processes.get(&pid)?;
        
        let should_enqueue = process.with_thread_mut(&tid, |thread| {
            let t = thread?;
            
            match t.status {
                ThreadStatus::Sleeping | ThreadStatus::SleepingNoDisturb => {
                    t.status = ThreadStatus::Waking;
                    Some(true)
                }
                ThreadStatus::Spawned => {
                    Some(false)
                }
                _ => None,
            }
        })?;
        
        if should_enqueue {
            self.blocked.remove(&(pid, tid));
            self.ready.push_back((pid, tid));
        }
        
        Some(())
    }

    pub fn prioritize(&mut self, pid: ProcessID, tid: ThreadID) -> bool {
        if self.current == Some((pid, tid)) {
            return true;
        }
        
        if let Some(idx) = self.ready.iter().position(|&x| x == (pid, tid)) {
            self.ready.remove(idx);
            self.ready.push_front((pid, tid));
            true
        } else {
            false
        }
    }
    
    pub fn prioritize_thread(&mut self, tid: ThreadID) -> bool {
        if let Some((_, current_tid)) = self.current {
            if current_tid == tid {
                return true;
            }
        }
        
        for (pid, t) in self.ready.iter() {
            if *t == tid {
                let pid = *pid;
                return self.prioritize(pid, tid);
            }
        }
        
        false
    }

    fn tick(&mut self) -> bool {
        let Some((pid, tid)) = self.current else {
            return !self.ready.is_empty() || self.idle.is_some();
        };

        let Some(process) = self.processes.get(&pid) else {
            return true;
        };

        process.with_thread_mut(&tid, |thread| {
            match thread {
                Some(t) if t.status == ThreadStatus::Dead 
                        || t.status == ThreadStatus::Sleeping 
                        || t.status == ThreadStatus::SleepingNoDisturb => {
                    true
                }
                Some(t) if t.initialised && t.quantum > 0 => {
                    t.quantum -= 1;
                    false
                }
                Some(t) => {
                    t.quantum = t.maxQuantum;
                    true
                }
                None => true,
            }
        })
    }

    fn switch_to_next(&mut self) -> Option<(ProcessID, ThreadID)> {
        if let Some((pid, tid)) = self.current.take() {
            if !self.blocked.contains(&(pid, tid)) {
                let is_runnable = self.processes.get(&pid)
                    .map(|p| p.with_thread_mut(&tid, |t| {
                        t.map(|t| !matches!(t.status, ThreadStatus::Dead | ThreadStatus::Sleeping | ThreadStatus::SleepingNoDisturb))
                            .unwrap_or(false)
                    }))
                    .unwrap_or(false);
                
                if is_runnable {
                    self.ready.push_back((pid, tid));
                }
            }
        }

        while let Some((pid, tid)) = self.ready.pop_front() {
            let Some(process) = self.processes.get(&pid) else {
                continue;
            };

            let is_runnable = process.with_thread_mut(&tid, |thread| {
                match thread {
                    Some(t) => {
                        if t.status == ThreadStatus::Waking {
                            t.status = ThreadStatus::Spawned;
                        }
                        matches!(t.status, ThreadStatus::Spawned)
                    }
                    None => false,
                }
            });

            if is_runnable {
                self.current = Some((pid, tid));
                return Some((pid, tid));
            }
        }

        self.current = self.idle;
        self.idle
    }

    pub fn switchTask(
        &mut self,
        saved_regs: *mut GPRegisters,
        frame: *const InterruptFrame,
    ) -> *mut GPRegisters {
        let frame = unsafe { &*frame };

        if !self.tick() {
            return saved_regs;
        }

        if let Some((pid, tid)) = self.current {
            if let Some(process) = self.processes.get(&pid) {
                process.with_thread_mut(&tid, |thread| {
                    if let Some(current) = thread {
                        // Save CPU State
                        current.gpRegisters = unsafe { *saved_regs };
                        current.iFrame = *frame;

                        // Save extended state (FPU/SSE/AVX)
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
                    }
                });
            }
        }

        let Some((next_pid, next_tid)) = self.switch_to_next() else {
            return saved_regs;
        };

        let Some(process) = self.processes.get(&next_pid) else {
            return saved_regs;
        };
        
        // Extract context and handle XSAVE resize atomically within the closure
        let ctx_result = process.with_thread_mut(&next_tid, |thread| {
            let next = thread?;
            
            if !next.initialised {
                next.initialised = true;
            }
            
            // Handle XSAVE resize if needed
            if get_fpu_mechanism() == FpuSaveMechanism::XSave {
                if let Some(newSize) = Self::getXAreaSize(next.xFeatures) {
                    if newSize != next.xAreaSize {
                        // Perform resize - allocate new, copy, deallocate old, update thread
                        let new_ptr = unsafe {
                            let Ok(new_layout) = Layout::from_size_align(newSize as usize, next.xAreaAlign as usize) else {
                                return None; // Invalid layout
                            };
                            let new_ptr = alloc(new_layout);
                            if new_ptr.is_null() {
                                return None; // Allocation failed
                            }
                            core::ptr::write_bytes(new_ptr, 0, newSize as usize);
                            
                            if let Some(old_ptr) = next.xAreaPtr {
                                let copy_size = core::cmp::min(next.xAreaSize, newSize) as usize;
                                core::ptr::copy_nonoverlapping(old_ptr, new_ptr, copy_size);
                                if let Ok(old_layout) = Layout::from_size_align(next.xAreaSize as usize, next.xAreaAlign as usize) {
                                    dealloc(old_ptr, old_layout);
                                }
                            } else {
                                // Initialize defaults if fresh
                                *(new_ptr as *mut u16) = 0x037F;
                                *(new_ptr.add(24) as *mut u32) = 0x1F80;
                            }
                            new_ptr
                        };
                        next.xAreaPtr = Some(new_ptr);
                        next.xAreaSize = newSize;
                    }
                }
            }
            
            Some(ThreadContext {
                cr3: next.cr3,
                gpRegisters: next.gpRegisters,
                iFrame: next.iFrame,
                xAreaPtr: next.xAreaPtr,
                xFeatures: next.xFeatures,
            })
        });

        let Some(ctx) = ctx_result else {
            return saved_regs;
        };
        
        let (currentCR3, flags) = Cr3::read();
        if currentCR3 != ctx.cr3 {
            unsafe { Cr3::write(ctx.cr3, flags); }
        }

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
                    xsetbv0(ctx.xFeatures.to_u64());
                    core::arch::x86_64::_xrstor64(ptr, ctx.xFeatures.to_u64());
                },
                FpuSaveMechanism::None => {},
            }
        }

        regs_ptr
    }
    
    /// Calculate XSAVE area size for given features.
    /// Returns None if features are unsupported.
    fn getXAreaSize(xFeatures: XFeatures) -> Option<u32> {
        unsafe {
            // Validate features are supported
            let (eax, _, _, edx) = CPUID(0xD, 0);
            let supported = (edx as u64) << 32 | eax as u64;
            if (xFeatures.to_u64() & !supported) != 0 {
                return None; // Unsupported features requested
            }

            let current_xcr0 = xgetbv0();
            xsetbv0(xFeatures.to_u64());
            let size_needed = CPUID(0xD, 0).1;
            xsetbv0(current_xcr0);

            Some(size_needed)
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

    let res = scheduler.switchTask(savedRegs, frame);

    kernelContext()
        .apic
        .get()
        .expect("APIC not initialized.")
        .notifyEOI();
    
    res
} 