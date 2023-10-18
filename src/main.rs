#![allow(non_snake_case)]
#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(panic_internals)]
#![feature(fmt_internals)]
#![feature(const_mut_refs)]
#![feature(abi_x86_interrupt)]
#![feature(lang_items)]

use bootloader::{entry_point, BootInfo};
use core::panic::{Location, PanicInfo};
use rust_OS::kernel::RTC::{waitSeconds, waitTicks};
use rust_OS::multitasking::preemptive::thread::Thread;
use x86_64::VirtAddr;

use rust_OS::fs::disk::floppy;
use rust_OS::kernel::{gdt, interrupts, RTC};
use rust_OS::mem::{allocator, memory, memory::BootInfoFrameAllocator, stack};
use rust_OS::multitasking::cooperative::{executor::Executor, Task};
use rust_OS::multitasking::preemptive::{ThreadID, SCHEDULER};
use rust_OS::println;
use rust_OS::tasks::keyboard;

extern crate alloc;

entry_point!(kMain);

fn kMain(bootInfo: &'static BootInfo) -> ! {
    println!("Hello World{}", "!");

    init();
    let physMemOffset = VirtAddr::new(bootInfo.physical_memory_offset);
    let mut mapper = unsafe { memory::init(physMemOffset) };
    let mut frameAllocator = unsafe { BootInfoFrameAllocator::init(&bootInfo.memory_map) };

    allocator::initHeap(&mut mapper, &mut frameAllocator).expect("heap initialization failed");

    unsafe {
        RTC::readRTC();
    }

    unsafe {
        let func = || {
            let a = 5;
            println!("Hello from thread! number: {}", a);
        };

        let func2 = || {
            let a = 96;
            loop {
                println!("Hello from thread2! number: {}", a);
            }
        };

        let mut thread = Thread::new(func, &mut mapper, &mut frameAllocator);
        let mut thread2 = Thread::new(func2, &mut mapper, &mut frameAllocator);
        thread.spawn();
        thread2.spawn();
    };

    loop {
        x86_64::instructions::hlt();
    }

    // let mut executor = Executor::new();
    // executor.spawn(Task::new(exampleTask()));
    // executor.spawn(Task::new(secTest(10)));
    // executor.spawn(Task::new(keyboard::printKeypresses()));
    // executor.spawn(Task::new(tickTest(200))); // 5ms per tick
    // executor.spawn(Task::new(floppy::detectFloppyDrives()));
    // executor.run();
}

async fn asyncNumber() -> u32 {
    42
}

async fn exampleTask() {
    let number = asyncNumber().await;
    println!("async number: {}", number);
}

async fn tickTest(ticks: u32) {
    println!("Ticks: {} begun", ticks);
    waitTicks(ticks).await;
    println!("Ticks: {} finished", ticks);
}

async fn secTest(secs: u32) {
    println!("Seconds: {} begun", secs);
    waitSeconds(secs).await;
    println!("Seconds: {} finished", secs);
}

#[panic_handler]
fn kpanic(info: &PanicInfo) -> ! {
    println!("{}", info);
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
        kpanic(&PanicInfo::internal_constructor(
            Some(&core::fmt::Arguments::new_v1(
                &["Error initializing keyboard"],
                &[],
            )),
            &Location::caller(),
            false,
        ));
    }

    RTC::initRTC();

    unsafe { interrupts::PICS.lock().initialize() };

    x86_64::instructions::interrupts::enable();
}

#[alloc_error_handler]
fn allocErrorHandler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout)
}
