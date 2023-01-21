#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![feature(custom_test_frameworks)]
#![test_runner(rust_OS::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]

use lazy_static::lazy_static;
use x86_64::structures::idt::InterruptDescriptorTable;
use rust_OS::{exit_qemu, QemuExitCode, serial_println, serial_print};
use x86_64::structures::idt::InterruptStackFrame;
use core::panic::PanicInfo;

extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_print!("stackOverflow::stackOverflow...\t");

    rust_OS::gdt::init();
    initTestIDT();

    // trigger a stack overflow
    stackOverflow();

    panic!("Execution continued after stack overflow");
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    rust_OS::testPanicHandler(_info)
}

#[allow(unconditional_recursion)]
fn stackOverflow() {
    stackOverflow(); // for each recursion, the return address is pushed
    volatile::Volatile::new(0).read(); // prevent tail recursion optimizations
}

lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
               .set_handler_fn(test_double_fault_handler)
               .set_stack_index(rust_OS::gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt
    };
}

pub fn initTestIDT() {
    TEST_IDT.load();
}