pub mod gdt;
pub mod interrupts;
//pub mod vgaBuffer;
pub mod AdvancedPic;
pub mod RTC;
pub mod binIO;
pub mod framebuffer;
pub mod kacpi;

use crate::mem::allocator;
use crate::mem::allocator::MultiHeapAllocator;
use crate::mem::memory::BootInfoFrameAllocator;
use bootloader_x86_64_common::logger::LockedLogger;
use core::fmt::Debug;
use linked_list_allocator::{Heap, LockedHeap};
use once_cell::sync::OnceCell;
use spin::Mutex;
use x86_64::structures::paging::OffsetPageTable;

#[global_allocator]
pub static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

static KERNEL_CONTEXT: OnceCell<KernelContext> = OnceCell::new();
pub struct KernelContext {
    pub logger: OnceCell<LockedLogger>,
    pub mapper: OnceCell<OffsetPageTable<'static>>,
    pub frameAllocator: OnceCell<BootInfoFrameAllocator>,
    pub heap_manager: OnceCell<Mutex<MultiHeapAllocator>>,
    pub frameBuffer: OnceCell<framebuffer::FrameBufferEditor>,
    pub apic: OnceCell<AdvancedPic::AdvancedPic>,
    pub constants: KernelConstants,
}

#[derive(Debug)]
pub struct KernelConstants {
    pub ACPI_INTERRUPT_MODEL: OnceCell<acpi::InterruptModel<'static, Heap>>,
    pub ACPI_PROCESSOR_INFO: OnceCell<acpi::platform::ProcessorInfo<'static, Heap>>,
}

pub fn initKernelContext() {
    KERNEL_CONTEXT
        .set(KernelContext {
            logger: OnceCell::new(),
            mapper: OnceCell::new(),
            frameAllocator: OnceCell::new(),
            heap_manager: OnceCell::new(),
            frameBuffer: OnceCell::new(),
            apic: OnceCell::new(),
            constants: KernelConstants {
                ACPI_INTERRUPT_MODEL: OnceCell::new(),
                ACPI_PROCESSOR_INFO: OnceCell::new(),
            },
        })
        .expect("Kernel Context already initialized, did you mean to get and set a resource?");
}

pub fn kernelContext() -> &'static KernelContext {
    KERNEL_CONTEXT
        .get()
        .expect("Kernel Context not initialized")
}

pub fn setKernelLogger(logger: LockedLogger) -> Option<&'static LockedLogger> {
    if let Err(_) = kernelContext().logger.set(logger) {
        log::error!("Logger already initialized.");
    }

    Some(kernelContext().logger.get().unwrap())
}

pub fn setKernelMapper(mapper: OffsetPageTable<'static>) {
    kernelContext()
        .mapper
        .set(mapper)
        .expect("Memory Mapper already initialized");
}

pub fn setKernelFrameAllocator(frameAllocator: BootInfoFrameAllocator) {
    kernelContext()
        .frameAllocator
        .set(frameAllocator)
        .expect("Frame Allocator already initialized");
}

pub fn setKernelHeapManager(heap_manager: MultiHeapAllocator) {
    kernelContext()
        .heap_manager
        .set(Mutex::new(heap_manager))
        .expect("Heap Manager already initialized");
}

pub fn setKernelFrameBuffer(frameBuffer: framebuffer::FrameBufferEditor) {
    kernelContext()
        .frameBuffer
        .set(frameBuffer)
        .expect("Frame Buffer already initialized");
}

pub fn setKernelAPIC(apic: AdvancedPic::AdvancedPic) {
    kernelContext()
        .apic
        .set(apic)
        .expect("APIC already initialized");
}

impl Debug for KernelContext {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("KernelContext")
            .field("logger", &"LockedLogger")
            .field("mapper", &self.mapper)
            .field("frameAllocator", &self.frameAllocator)
            .field("heap_manager", &self.heap_manager)
            .field("frameBuffer", &self.frameBuffer)
            .field("apic", &self.apic)
            .field("constants", &self.constants)
            .finish()
    }
}
