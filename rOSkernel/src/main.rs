#![allow(non_snake_case)]
#![allow(named_asm_labels)]
#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]

extern crate alloc;
use acpi::AcpiTables;
use bootloader_api::config::{BootloaderConfig, Mapping};
use bootloader_api::info::{FrameBuffer, FrameBufferInfo, MemoryRegions};
use bootloader_api::BootInfo;
use bootloader_x86_64_common::logger::LockedLogger;
use core::alloc::Layout;
use core::panic::PanicInfo;
use linked_list_allocator::Heap;
use log::log;
use rOSkernel::fs::disk::floppy;
use rOSkernel::kernel::framebuffer::FrameBufferEditor;
use rOSkernel::kernel::interrupts::APIC_BASE;
use rOSkernel::kernel::kacpi::ACPIHandler;
use rOSkernel::kernel::AdvancedPic::AdvancedPic;
use rOSkernel::kernel::{
    gdt, interrupts, kernelContext, setKernelAPIC, setKernelFrameAllocator, setKernelFrameBuffer, setKernelHeapManager,
    setKernelLogger, setKernelMapper, HEAP_ALLOCATOR, RTC,
};
use rOSkernel::mem::allocator::MultiHeapAllocator;
use rOSkernel::mem::{allocator, memory, memory::BootInfoFrameAllocator};
use rOSkernel::multitasking::cooperative::executor::Executor;
use rOSkernel::multitasking::cooperative::Task;
use rOSkernel::tasks::keyboard;
use rOSkernel::tasks::keyboard::printKeypresses;
use x86_64::structures::paging::OffsetPageTable;
use x86_64::VirtAddr;

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

bootloader_api::entry_point!(kMain, config = &BOOTLOADER_CONFIG);

#[panic_handler]
fn kPanic(info: &PanicInfo) -> ! {
    unsafe {
        bootloader_x86_64_common::logger::LOGGER
            .get()
            .map(|l: &LockedLogger| l.force_unlock())
    };
    log::error!("{}", info);

    x86_64::instructions::interrupts::disable();
    loop {
        x86_64::instructions::nop();
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("Allocation error: {:?}", layout)
}

fn kMain(bootInfo: &'static mut BootInfo) -> ! {
    let physMemOffset = bootInfo.physical_memory_offset.into_option().unwrap();
    memory::PHYSICAL_MEMORY_OFFSET.get_or_init(|| physMemOffset);

    // Instead of borrowing bootInfo.framebuffer multiple times, get a raw pointer.
    let fbPtr = bootInfo.framebuffer.as_mut().unwrap() as *mut FrameBuffer;
    let fbInfo = unsafe { (*fbPtr).info() };
    let fbBytes = unsafe { (&mut *fbPtr).buffer_mut() };
    initLogger(fbBytes, fbInfo);

    log::info!("Bootloader framebuffer info: {:?}", fbInfo);
    loop {}

    setKernelFrameBuffer({
        let fb = unsafe { &mut *fbPtr };
        FrameBufferEditor::new(fb, fbInfo)
    });

    log::info!("about to set up gdt and idt");

    gdt::init();
    interrupts::initIDT();
    // unsafe { interrupts::PICS.lock().initialize() }; // if not using APIC
    x86_64::instructions::interrupts::enable();
    // initialise PIT
    RTC::initRTC();

    let (mut mapper, mut frameAllocator) = initMemory(
        memory::PHYSICAL_MEMORY_OFFSET.get_copy().unwrap(),
        &bootInfo.memory_regions,
    );

    log::info!("about to set up ACPI");

    let acpiTables = unsafe {
        AcpiTables::from_rsdp(
            ACPIHandler,
            bootInfo.rsdp_addr.into_option().unwrap() as usize,
        )
        .expect("TODO: panic message")
    };
    let fadt = acpiTables.find_table::<acpi::fadt::Fadt>().unwrap();
    let dsdt = acpiTables.dsdt().unwrap();
    let madtPhysMap = acpiTables.find_table::<acpi::madt::Madt>().unwrap();
    let madt = madtPhysMap.get();
    let mut heapMgr = kernelContext().heap_manager.get().unwrap().lock();
    let acpiAllocator: Heap = unsafe {
        let (start, len) = heapMgr
            .init_heap(&mut mapper, &mut frameAllocator, 1024 * 1024)
            .unwrap();
        Heap::new(start.as_mut_ptr(), len as usize)
    };
    let madtInfo = madt.parse_interrupt_model_in(acpiAllocator).unwrap();

    kernelContext()
        .constants
        .ACPI_PROCESSOR_INFO
        .get_or_init(|| madtInfo.1.unwrap());

    kernelContext()
        .constants
        .ACPI_INTERRUPT_MODEL
        .get_or_init(|| madtInfo.0);

    // now initialize kernelContext mapper and frame allocator
    setKernelMapper(mapper);
    setKernelFrameAllocator(frameAllocator);

    if let Err(e) = keyboard::keyboardInitialize() {
        panic!("Failed to initialize keyboard: {:?}", e);
    }

    AdvancedPic::init();

    log::info!("Hello from kernel!");

    // extern "C" fn t1(context: &mut Context) {
    //     unsafe {
    //         core::arch::asm!("cli");
    //     }
    //     return;
    // }

    // unsafe {
    //     // execute t1 in ring 3
    //     core::arch::asm!("
    //     mov ax, {0:x}
    //     mov ds, ax
    //     mov es, ax
    //     mov fs, ax
    //     mov gs, ax

    //     mov rdi, {3:r}

    //     push {0:r}
    //     mov rax, {2:r}
    //     push rax
    //     pushf
    //     push {1:r}
    //     push {2:r}
    //     iretq",
    //     in(reg) GDT.1.userDataSelector.0,
    //     in(reg) GDT.1.userCodeSelector.0,
    //     in(reg) t1 as *const () as u64
    //     in(reg) context as *const Context as u64,
    //     );
    // }

    async fn secTest(secs: u32) {
        log::info!("Seconds: {} begun", secs);
        RTC::waitSeconds(secs).await;
        log::info!("Seconds: {} finished", secs);
    }

    async fn tickTest(ticks: u32) {
        log::info!("Ticks: {} begun", ticks);
        RTC::waitTicks(ticks).await;
        log::info!("Ticks: {} finished", ticks);
    }

    let mut executor = Executor::new();
    //  executor.spawn(Task::new(exampleTask()));
    executor.spawn(Task::new(secTest(10)));
    executor.spawn(Task::new(printKeypresses()));
    executor.spawn(Task::new(tickTest(200))); // 5ms per tick
    executor.spawn(Task::new(floppy::detectFloppyDrives()));
    executor.run();

    // unsafe {
    //     extern "C" fn func() {
    //         let a = 5;
    //         loop {
    //             log::info!("Hello from thread! number: {}", a);
    //         }
    //     }
    //
    //     extern "C" fn func2() {
    //         let a = 96;
    //         loop {
    //             log::info!("Hello from thread2! number: {}", a);
    //         }
    //     }
    //
    //     let thread = Thread::new(func, &mut mapper, &mut frameAllocator);
    //     let thread2 = Thread::new(func2, &mut mapper, &mut frameAllocator);
    //     thread.spawn();
    //     thread2.spawn();
    // };
    //
    // loop {
    //     x86_64::instructions::hlt();
    // }
}

pub fn initMemory(
    physicalMemoryOffset: u64,
    memoryRegions: &'static MemoryRegions,
) -> (OffsetPageTable<'static>, BootInfoFrameAllocator) {
    let virtMemOffset = VirtAddr::new(physicalMemoryOffset);
    let mut mapper = unsafe { memory::init(virtMemOffset) };
    let mut frameAllocator = unsafe { BootInfoFrameAllocator::init(memoryRegions) };

    // initialize heap with desired size using a multi-heap allocator
    setKernelHeapManager(allocator::MultiHeapAllocator::new());
    let heap_size: u64 = 100 * 1024;
    let (heap_start, heap_size) = kernelContext()
        .heap_manager
        .get()
        .unwrap()
        .lock()
        .init_heap(&mut mapper, &mut frameAllocator, heap_size)
        .expect("heap initialization failed");
    // initialize global allocator with allocated heap region
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(heap_start.as_u64() as *mut u8, heap_size as usize);
    }

    (mapper, frameAllocator)
}

pub fn initLogger(buffer: &'static mut [u8], info: FrameBufferInfo) {
    let logger = setKernelLogger(LockedLogger::new(buffer, info, true, true));
    log::set_logger(logger.unwrap()).expect("initLogger failed");
    log::set_max_level(log::LevelFilter::Trace);
}
