#![no_std]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(alloc_error_handler)]
#![feature(panic_internals)]
#![feature(fmt_internals)]
#![feature(const_mut_refs)]
#![feature(abi_x86_interrupt)]
#![feature(lang_items)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

#[cfg(test)]
use bootloader::{entry_point, BootInfo};

#[cfg(test)]
use task::keyboard;


extern crate alloc;

pub mod kernel;
pub mod debug;
pub mod mem;
pub mod task;
pub mod fs;
pub mod util;

#[cfg(test)]
entry_point!(test_kMain);

#[cfg(test)]
fn test_kMain(_boot_info: &'static BootInfo) -> ! {
    testInit();
    test_main();
    hltLoop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    testPanicHandler(info)
}

#[cfg(test)]
#[alloc_error_handler]
fn allocErrorHandler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout)
}

#[cfg(test)]
pub fn testInit() {
    use crate::kernel::{interrupts, gdt};

    gdt::init();
    interrupts::initIDT();
    unsafe { interrupts::PICS.lock().initialize() };
    keyboard::keyboardInitialize().unwrap();
    x86_64::instructions::interrupts::enable();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

pub trait Testable {
    fn run(&self);
}

impl<T> Testable for T where T: Fn(), {
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn testPanicHandler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hltLoop();
}

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }

    exit_qemu(QemuExitCode::Success);
}

#[test_case]
fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}

pub fn hltLoop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}