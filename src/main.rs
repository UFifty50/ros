#![allow(non_snake_case)]
#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(panic_internals)]
#![feature(fmt_internals)]
#![feature(const_mut_refs)]
#![feature(abi_x86_interrupt)]
#![feature(lang_items)]

use core::panic::{PanicInfo, Location};
use bootloader::{BootInfo, entry_point};
use x86_64::VirtAddr;
use rust_OS::{println, gdt, interrupts, memory, allocator};
use rust_OS::task::executor::Executor;
use rust_OS::memory::BootInfoFrameAllocator;
use rust_OS::task::{Task, keyboard};

extern crate alloc;

entry_point!(kMain);

fn kMain(bootInfo: &'static BootInfo) -> ! {
    println!("Hello World{}", "!");

    init();
    let physMemOffset = VirtAddr::new(bootInfo.physical_memory_offset);
    let mut mapper = unsafe { memory::init(physMemOffset) };
    let mut frameAllocator = unsafe { BootInfoFrameAllocator::init(&bootInfo.memory_map) };
    
    allocator::initHeap(&mut mapper, &mut frameAllocator).expect("heap initialization failed");

    let mut executor = Executor::new();
    executor.spawn(Task::new(exampleTask()));
    executor.spawn(Task::new(keyboard::printKeypresses()));
    executor.run();

    println!("It did not crash!");
    hltLoop();
}

async fn asyncNumber() -> u32 {
    42
}

async fn exampleTask() {
    let number = asyncNumber().await;
    println!("async number: {}", number);
}

#[panic_handler]
fn kpanic(info: &PanicInfo) -> ! {
    println!("{}", info);
    hltLoop();
}

pub fn init() {
    gdt::init();
    interrupts::initIDT();
    let init = keyboard::keyboardInitialize();
    if Result::is_err(&init) {
        kpanic(&PanicInfo::internal_constructor(
            Some(&core::fmt::Arguments::new_v1(&["Error initializing keyboard"], &[])),
            &Location::caller(),
            false
        ));
    }
    unsafe { interrupts::PICS.lock().initialize() };

    x86_64::instructions::interrupts::enable();
}

#[alloc_error_handler]
fn allocErrorHandler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout)
}

pub fn wait(cycles: i32) {
    for _ in 0..=cycles {
        x86_64::instructions::hlt();
    }
}

pub fn hltLoop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
