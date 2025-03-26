use core::sync::atomic::{AtomicU32, Ordering};

use lazy_static::lazy_static;
use pic8259::ChainedPics;
use ps2::Controller;
use spin::Mutex;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use x86_64::PrivilegeLevel;

use crate::kernel::{binIO, gdt, RTC};
use crate::multitasking::preemptive::SCHEDULER;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.non_maskable_interrupt
            .set_handler_fn(nonMaskableInterruptHandler);
        idt.breakpoint.set_handler_fn(breakpointHandler);
        idt.invalid_opcode.set_handler_fn(invalidOpcodeHandler);
        idt.segment_not_present
            .set_handler_fn(segmentNotPresentHandler);
        idt.stack_segment_fault
            .set_handler_fn(stackSegmentFaultHandler);
        idt.invalid_tss.set_handler_fn(invalidTSSHandler);
        idt[InterruptIndex::Timer as u8].set_handler_fn(timerInterruptHandler);
        idt[InterruptIndex::Keyboard as u8].set_handler_fn(keyboardInterruptHandler);
        idt[InterruptIndex::Floppy as u8].set_handler_fn(floppyInterruptHandler);
        idt[InterruptIndex::RealTimeClock as u8].set_handler_fn(realTimeClockInterruptHandler);
        idt[InterruptIndex::SystemCall as u8]
            .set_handler_fn(breakpointHandler)
            .disable_interrupts(false)
            .set_privilege_level(PrivilegeLevel::Ring3);
        unsafe {
            idt.double_fault
                .set_handler_fn(doubleFaultHandler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX as u16);

            idt.page_fault
                .set_handler_fn(pageFaultHandler)
                .set_stack_index(gdt::PAGE_FAULT_IST_INDEX as u16);

            idt.general_protection_fault
                .set_handler_fn(GPFaultHandler)
                .set_stack_index(gdt::GENERAL_FAULT_IST_INDEX as u16);
        }

        idt
    };
    pub static ref CONTROLLER: Mutex<Controller> =
        unsafe { Mutex::new(Controller::with_timeout(50000)) };
}

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

pub(crate) static TICK_COUNTER: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
    Floppy = PIC_1_OFFSET + 6,
    RealTimeClock = PIC_2_OFFSET,
    SystemCall = 0xAA - PIC_1_OFFSET,
}

pub fn initIDT() {
    IDT.load();
}

extern "x86-interrupt" fn nonMaskableInterruptHandler(stackFrame: InterruptStackFrame) {
    log::error!("EXCEPTION: NON MASKABLE INTERRUPT\n{:#?}", stackFrame);
    loop {
        x86_64::instructions::hlt();
    }
}

extern "x86-interrupt" fn breakpointHandler(stackFrame: InterruptStackFrame) {
    log::error!("EXCEPTION: BREAKPOINT\n{:#?}", stackFrame);
}

extern "x86-interrupt" fn invalidOpcodeHandler(stackFrame: InterruptStackFrame) {
    log::error!("EXCEPTION: INVALID OPCODE\n{:#?}", stackFrame);
}

extern "x86-interrupt" fn pageFaultHandler(
    stackFrame: InterruptStackFrame,
    errCode: PageFaultErrorCode,
) {
    log::error!("EXCEPTION: PAGE FAULT");
    log::error!("Accessed Address: {:?}", Cr2::read());
    log::error!("{:#?}", stackFrame);
    log::error!("Error Code: {:?}", errCode);
    loop {
        x86_64::instructions::hlt();
    }
}

extern "x86-interrupt" fn GPFaultHandler(stackFrame: InterruptStackFrame, errCode: u64) {
    log::error!(
        "EXCEPTION: GENERAL PROTECTION FAULT\n{:#?}\nError Code: {}",
        stackFrame, errCode
    );
    loop {
        x86_64::instructions::hlt();
    }
}

extern "x86-interrupt" fn segmentNotPresentHandler(stackFrame: InterruptStackFrame, errCode: u64) {
    log::error!(
        "EXCEPTION: SEGMENT NOT PRESENT\n{:#?}\nError Code: {}",
        stackFrame, errCode
    );
    loop {
        x86_64::instructions::hlt();
    }
}

extern "x86-interrupt" fn stackSegmentFaultHandler(stackFrame: InterruptStackFrame, errCode: u64) {
    log::error!(
        "EXCEPTION: STACK SEGMENT FAULT\n{:#?}\nError Code: {}",
        stackFrame, errCode
    );
    loop {
        x86_64::instructions::hlt();
    }
}

extern "x86-interrupt" fn invalidTSSHandler(stackFrame: InterruptStackFrame, _errCode: u64) {
    log::error!("EXCEPTION: INVALID TSS\n{:#?}", stackFrame);
    loop {
        x86_64::instructions::hlt();
    }
}

extern "x86-interrupt" fn timerInterruptHandler(_stackFrame: InterruptStackFrame) {
    log::trace!(".");
    TICK_COUNTER.fetch_add(1, Ordering::Relaxed);
    
    x86_64::instructions::interrupts::disable();
    SCHEDULER.lock().switchTask();
    x86_64::instructions::interrupts::enable();

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer as u8);
    }
}

extern "x86-interrupt" fn keyboardInterruptHandler(_stackFrame: InterruptStackFrame) {
    let mut controller = unsafe {
        CONTROLLER.force_unlock();
        CONTROLLER.lock()
    };

    let scanread = controller.read_data();
    if let Ok(mut scancode) = scanread {
        scancode = scanread.unwrap();
        crate::tasks::keyboard::addScancode(scancode);
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard as u8);
    }
}

extern "x86-interrupt" fn floppyInterruptHandler(_stackFrame: InterruptStackFrame) {
    log::trace!("F");
    // TODO: call fs::floppy::<methods to read/write floppy>
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Floppy as u8);
    }
}

extern "x86-interrupt" fn realTimeClockInterruptHandler(_stackFrame: InterruptStackFrame) {
    unsafe {
        binIO::out8(0x70, 0x0C);
        binIO::in8(0x71);
        RTC::readRTC();

        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::RealTimeClock as u8);
    }
}

extern "x86-interrupt" fn doubleFaultHandler(stackFrame: InterruptStackFrame, _errCode: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stackFrame);
}
