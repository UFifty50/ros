pub mod gdt;
pub mod interrupts;
//pub mod vgaBuffer;
pub mod timer;
pub mod AdvancedPic;
pub mod RTC;
pub mod binIO;
pub mod framebuffer;
pub mod kacpi;

use crate::mem::allocator::HeapRegionAllocator;
use crate::mem::memory::BootInfoFrameAllocator;
use crate::util::wrappers::XFeatures;
use bootloader_x86_64_common::logger::LockedLogger;
use core::fmt::Debug;
use once_cell::sync::OnceCell;
use spin::Mutex;
use x86_64::structures::paging::OffsetPageTable;
use crate::mem::heap::Heap;

// See src/mem/mod.rs for #[global_allocator]

static KERNEL_CONTEXT: OnceCell<KernelContext> = OnceCell::new();
pub struct KernelContext {
    pub logger: OnceCell<LockedLogger>,
    pub mapper: OnceCell<Mutex<OffsetPageTable<'static>>>,
    pub frameAllocator: OnceCell<Mutex<BootInfoFrameAllocator>>,
    pub heapRegionAllocator: OnceCell<Mutex<HeapRegionAllocator>>,
    pub frameBuffer: OnceCell<framebuffer::FrameBufferEditor>,
    pub apic: OnceCell<AdvancedPic::AdvancedPic>,
    pub timerQueue: OnceCell<timer::TimerQueue>,
    pub constants: KernelConstants,
}

#[derive(Debug)]
pub struct KernelConstants {
    pub ACPI_INTERRUPT_MODEL: OnceCell<acpi::InterruptModel<'static, Heap>>,
    pub ACPI_PROCESSOR_INFO: OnceCell<acpi::platform::ProcessorInfo<'static, Heap>>,
    pub SUPPORTED_XFEATURES: OnceCell<XFeatures>,
}

pub fn initKernelContext() {
    KERNEL_CONTEXT
        .set(KernelContext {
            logger: OnceCell::new(),
            mapper: OnceCell::new(),
            frameAllocator: OnceCell::new(),
            heapRegionAllocator: OnceCell::new(),
            frameBuffer: OnceCell::new(),
            apic: OnceCell::new(),
            timerQueue: OnceCell::new(),
            constants: KernelConstants {
                ACPI_INTERRUPT_MODEL: OnceCell::new(),
                ACPI_PROCESSOR_INFO: OnceCell::new(),
                SUPPORTED_XFEATURES: OnceCell::new(),
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

pub fn setKernelMapper(mapperMutex: Mutex<OffsetPageTable<'static>>) {
    kernelContext()
        .mapper
        .set(mapperMutex)
        .expect("Memory Mapper already initialized");
}

pub fn setKernelFrameAllocator(frameAllocatorMutex: Mutex<BootInfoFrameAllocator>) {
    kernelContext()
        .frameAllocator
        .set(frameAllocatorMutex)
        .expect("Frame Allocator already initialized");
}

pub fn setKernelHeapManager(heap_manager: HeapRegionAllocator) {
    kernelContext()
        .heapRegionAllocator
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

pub fn setKernelTimerQueue(timerQueue: timer::TimerQueue) {
    kernelContext()
        .timerQueue
        .set(timerQueue)
        .expect("Timer Queue already initialized");
}

impl Debug for KernelContext {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("KernelContext")
            .field("logger", &"LockedLogger")
            .field("mapper", &self.mapper)
            .field("frameAllocator", &self.frameAllocator)
            .field("heap_manager", &self.heapRegionAllocator)
            .field("frameBuffer", &self.frameBuffer)
            .field("apic", &self.apic)
            .field("timerQueue", &self.timerQueue)
            .field("constants", &self.constants)
            .finish()
    }
}
