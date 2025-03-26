#![allow(non_snake_case)]
#![allow(named_asm_labels)]
#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use bootloader_api::config::{BootloaderConfig, Mapping};
use bootloader_api::info::{FrameBuffer, FrameBufferInfo, MemoryRegions};
use bootloader_api::BootInfo;
use bootloader_x86_64_common::logger::LockedLogger;
use conquer_once::spin::OnceCell;
use x86_64::structures::paging::OffsetPageTable;
use core::prelude::rust_2024::alloc_error_handler;
use rOSkernel::kernel::framebuffer::{FrameBufferEditor, FRAMEBUFFER};
use x86_64::VirtAddr;
//use rOSkernel::kernel::RTC::{waitSeconds, waitTicks};
use rOSkernel::multitasking::preemptive::thread::Thread;
use rOSkernel::kernel::{gdt, interrupts, RTC};
use rOSkernel::mem::{allocator, memory, memory::BootInfoFrameAllocator};
use rOSkernel::tasks::keyboard;

extern crate alloc;


pub static LOGGER: OnceCell<LockedLogger> = OnceCell::uninit();
pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

bootloader_api::entry_point!(kMain, config = &BOOTLOADER_CONFIG);

fn kMain(bootInfo: &'static mut BootInfo) -> ! {
    // Instead of borrowing bootInfo.framebuffer multiple times, obtain a raw pointer.
    let fbPtr = bootInfo.framebuffer.as_mut().unwrap() as *mut FrameBuffer;
    let fbInfo = unsafe { (*fbPtr).info().clone() };

FRAMEBUFFER.get_or_init(|| {
         let fb = unsafe { &mut *fbPtr };
         FrameBufferEditor::new(fb, fbInfo)
    });
    
    let fbBytes = unsafe { (&mut *fbPtr).buffer_mut() };
    initLogger(fbBytes, fbInfo);

    init();
    let memoryOffset = bootInfo.physical_memory_offset.into_option().unwrap();
    let (mut mapper, mut frameAllocator) = initMemory(memoryOffset, &bootInfo.memory_regions);

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

    unsafe {
        extern "C" fn func() {
            let a = 5;
            loop {
                log::info!("Hello from thread! number: {}", a);
            }
        }

        extern "C" fn func2() {
            let a = 96;
            loop {
                log::info!("Hello from thread2! number: {}", a);
            }
        }

        let thread = Thread::new(func, &mut mapper, &mut frameAllocator);
        let thread2 = Thread::new(func2, &mut mapper, &mut frameAllocator);
        thread.spawn();
        thread2.spawn();
    };

    loop {
        x86_64::instructions::hlt();
    }

    //  let mut executor = Executor::new();
    //  executor.spawn(Task::new(exampleTask()));
    //  executor.spawn(Task::new(secTest(10)));
    //  executor.spawn(Task::new(keyboard::printKeypresses()));
    //  executor.spawn(Task::new(tickTest(200))); // 5ms per tick
    //  executor.spawn(Task::new(floppy::detectFloppyDrives()));
    //  executor.run();
}

// async fn asyncNumber() -> u32 {
//     42
// }

// async fn exampleTask() {
//     let number = asyncNumber().await;
//     println!("async number: {}", number);
// }

// async fn tickTest(ticks: u32) {
//     println!("Ticks: {} begun", ticks);
//     waitTicks(ticks).await;
//     println!("Ticks: {} finished", ticks);
// }

// async fn secTest(secs: u32) {
//     println!("Seconds: {} begun", secs);
//     waitSeconds(secs).await;
//     println!("Seconds: {} finished", secs);
// }

#[panic_handler]
fn kPanic(info: &PanicInfo) -> ! {
    log::error!("{}", info);
    x86_64::instructions::interrupts::disable();
    loop {
        x86_64::instructions::hlt();
    }
}

pub fn init() {
    gdt::init();
    interrupts::initIDT();

    let initKeyboard = keyboard::keyboardInitialize();
    if initKeyboard.is_err() {
        // kpanic(&PanicInfo::internal_constructor(
        //     Some(&core::fmt::Arguments::new_v1(
        //         &["Error initializing keyboard"],
        //         &[],
        //     )),
        //     &Location::caller(),
        //     false,
        //     false,
        // ));
    }

    RTC::initRTC();

    unsafe { interrupts::PICS.lock().initialize() };

    x86_64::instructions::interrupts::enable();
}

pub fn initMemory(memoryOffset: u64, memoryRegions: &'static MemoryRegions
) -> (OffsetPageTable<'static>, BootInfoFrameAllocator) {
    let physMemOffset = VirtAddr::new(memoryOffset);
    let mut mapper = unsafe { memory::init(physMemOffset) };
    let mut frameAllocator = unsafe { BootInfoFrameAllocator::init(memoryRegions) };
    allocator::initHeap(&mut mapper, &mut frameAllocator).expect("heap initialization failed");

    (mapper, frameAllocator)
}

pub fn initLogger(buffer: &'static mut [u8], info: FrameBufferInfo) {
    let logger = LOGGER.get_or_init(|| LockedLogger::new(buffer, info, true, false));
    log::set_logger(logger).expect("initLogger failed");
    log::set_max_level(log::LevelFilter::Trace);
}

#[alloc_error_handler]
fn allocErrorHandler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout)
}

